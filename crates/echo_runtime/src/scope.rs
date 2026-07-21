//! Scope-owned memory registries (ADR 0016).
//!
//! Every managed heap handle is registered to exactly one owning scope.
//! Promotion moves the ownership record; scope exit releases remaining owned
//! values. No tracing GC and no reference counts for aliases.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use crate::{
    header_at, HEAP_MAGIC, KIND_BYTES, KIND_FLOAT, KIND_FN, KIND_LIST, KIND_LOCATOR, KIND_RANGE,
    KIND_STRING, KIND_STRUCT,
};

/// Logical tombstone: handle may still be in memory until deferred free runs.
#[derive(Default)]
struct ScopeState {
    /// Stack of open scopes (top = innermost).
    stack: Vec<ScopeFrame>,
    /// handle → stack index of the owning frame (exactly one owner when live).
    /// Indices are invalidated on pop; exit path frees via the popped frame set.
    owner: HashMap<i64, usize>,
    /// Logically released; physical free may be deferred.
    dead: HashSet<i64>,
    /// Deferred physical frees (event-loop batching).
    deferred: Vec<i64>,
}

struct ScopeFrame {
    id: u32,
    owned: HashSet<i64>,
}

thread_local! {
    static STATE: RefCell<ScopeState> = RefCell::new(ScopeState::default());
}

/// Enter a scope (push). Compile-time `id`s are per-function; the same id may
/// re-enter across nested calls (each push is a distinct dynamic frame).
#[unsafe(no_mangle)]
pub extern "C" fn echo_runtime_scope_enter(id: i64) {
    let id = id as u32;
    STATE.with(|c| {
        let mut st = c.borrow_mut();
        st.stack.push(ScopeFrame {
            id,
            owned: HashSet::new(),
        });
    });
}

/// Register `handle` as owned by the **current** (innermost) scope.
/// No-op for 0 / non-heap immediates.
/// If already owned, leave ownership unchanged (alias-safe; no double free).
#[unsafe(no_mangle)]
pub extern "C" fn echo_runtime_scope_register(handle: i64) {
    if handle == 0 || !is_managed_handle(handle) {
        return;
    }
    STATE.with(|c| {
        let mut st = c.borrow_mut();
        // Address reuse after free: drop tombstone and re-home as a new live object.
        st.dead.remove(&handle);
        if st.owner.contains_key(&handle) {
            // Already owned (alias or re-register) — keep single owner.
            return;
        }
        let Some(idx) = st.stack.len().checked_sub(1) else {
            // No open scope: process root — leave unregistered (slice-1 safety).
            return;
        };
        st.stack[idx].owned.insert(handle);
        st.owner.insert(handle, idx);
    });
}

/// Move ownership of `handle` to an open scope `target_id` (must be on the stack).
/// Source scope no longer releases it.
///
/// When the same compile-time id is nested (re-entrant calls), promotion targets
/// the **innermost** open frame with that id (nearest enclosing match from the
/// top of the stack).
#[unsafe(no_mangle)]
pub extern "C" fn echo_runtime_scope_promote(handle: i64, target_id: i64) {
    if handle == 0 || !is_managed_handle(handle) {
        return;
    }
    let target_id = target_id as u32;
    STATE.with(|c| {
        let mut st = c.borrow_mut();
        if st.dead.contains(&handle) {
            // Allow promote of a logically released handle only if address was
            // reused; clear tombstone and re-home (slice-1 recovery).
            st.dead.remove(&handle);
        }
        // Innermost frame with this compile-time id.
        let Some(target_idx) = st.stack.iter().rposition(|f| f.id == target_id) else {
            eprintln!("echo: promote to non-open scope {target_id}");
            std::process::exit(1);
        };
        // Remove from whatever frame currently owns the handle.
        if let Some(&src_idx) = st.owner.get(&handle) {
            if src_idx == target_idx {
                return;
            }
            if src_idx < st.stack.len() {
                st.stack[src_idx].owned.remove(&handle);
            }
        } else {
            for f in st.stack.iter_mut() {
                f.owned.remove(&handle);
            }
        }
        st.stack[target_idx].owned.insert(handle);
        st.owner.insert(handle, target_idx);
    });
}

/// Remove ownership without free (e.g. return / transfer out of analysis).
#[unsafe(no_mangle)]
pub extern "C" fn echo_runtime_scope_disown(handle: i64) {
    if handle == 0 {
        return;
    }
    STATE.with(|c| {
        let mut st = c.borrow_mut();
        if let Some(idx) = st.owner.remove(&handle) {
            if idx < st.stack.len() {
                st.stack[idx].owned.remove(&handle);
            }
        } else {
            for f in st.stack.iter_mut() {
                f.owned.remove(&handle);
            }
        }
    });
}

/// Logical release of one value (immediate lightweight free when possible).
#[unsafe(no_mangle)]
pub extern "C" fn echo_runtime_scope_release(handle: i64) {
    if handle == 0 {
        return;
    }
    STATE.with(|c| {
        let mut st = c.borrow_mut();
        logical_release(&mut st, handle, /*defer_heavy*/ false);
    });
}

/// Enqueue physical destruction (batch drain later).
#[unsafe(no_mangle)]
pub extern "C" fn echo_runtime_scope_enqueue_release(handle: i64) {
    if handle == 0 {
        return;
    }
    STATE.with(|c| {
        let mut st = c.borrow_mut();
        logical_release(&mut st, handle, /*defer_heavy*/ true);
    });
}

/// Exit scope `id` (must be the current top). Releases every still-owned value.
#[unsafe(no_mangle)]
pub extern "C" fn echo_runtime_scope_exit(id: i64) {
    let id = id as u32;
    STATE.with(|c| {
        let mut st = c.borrow_mut();
        let Some(frame) = st.stack.pop() else {
            eprintln!("echo: scope_exit with empty stack");
            std::process::exit(1);
        };
        if frame.id != id {
            eprintln!(
                "echo: scope_exit id mismatch: expected {}, got {id}",
                frame.id
            );
            std::process::exit(1);
        }
        // Release in reverse insertion order approximation: iterate owned set.
        let owned: Vec<i64> = frame.owned.into_iter().collect();
        for h in owned {
            logical_release(&mut st, h, /*defer_heavy*/ false);
        }
    });
}

/// Drain deferred physical frees (call from event-loop idle points).
#[unsafe(no_mangle)]
pub extern "C" fn echo_runtime_scope_drain_deferred() {
    STATE.with(|c| {
        let mut st = c.borrow_mut();
        let batch = std::mem::take(&mut st.deferred);
        for h in batch {
            physical_free(h);
        }
    });
}

fn logical_release(st: &mut ScopeState, handle: i64, defer_heavy: bool) {
    if handle == 0 {
        return;
    }
    if !st.dead.insert(handle) {
        // Already logically dead — ignore (alias-safe).
        return;
    }
    st.owner.remove(&handle);
    // Scrub from every open frame (safe after scope_exit already popped its frame).
    for f in st.stack.iter_mut() {
        f.owned.remove(&handle);
    }
    // Slice-1: always enqueue for deferred free. Immediate free is still correct
    // for unique ownership, but incomplete promotion (conservative analysis gaps)
    // can free a handle that a later path reuses. Deferred free keeps logical
    // death while avoiding use-after-free until analysis is complete; drain is
    // available for event-loop points. Process exit reclaims the rest.
    st.deferred.push(handle);
    let _ = defer_heavy;
}

fn is_managed_handle(handle: i64) -> bool {
    // Prefer the live-set: never probe wild integers (ui64 digests, etc.) as
    // pointers — that segfaults in header_at.
    if crate::is_live_heap(handle) {
        return true;
    }
    false
}

/// Physical free by kind. Safe only after logical release.
fn physical_free(handle: i64) {
    if handle == 0 {
        return;
    }
    if !crate::is_live_heap(handle) {
        return;
    }
    let Some(h) = (unsafe { header_at(handle) }) else {
        crate::note_heap_free(handle);
        return;
    };
    let kind = unsafe { (*h).kind };
    // Reconstruct Box and drop.
    match kind {
        KIND_LIST => {
            let _ = unsafe { Box::from_raw(handle as *mut crate::EchoList) };
        }
        KIND_STRING => {
            let _ = unsafe { Box::from_raw(handle as *mut crate::EchoString) };
        }
        KIND_STRUCT => {
            let _ = unsafe { Box::from_raw(handle as *mut crate::EchoStruct) };
        }
        KIND_FLOAT => {
            let _ = unsafe { Box::from_raw(handle as *mut crate::EchoFloat) };
        }
        KIND_BYTES => {
            let _ = unsafe { Box::from_raw(handle as *mut crate::EchoBytes) };
        }
        KIND_LOCATOR => {
            let _ = unsafe { Box::from_raw(handle as *mut crate::EchoLocator) };
        }
        KIND_RANGE => {
            let _ = unsafe { Box::from_raw(handle as *mut crate::EchoRange) };
        }
        KIND_FN => {
            let _ = unsafe { Box::from_raw(handle as *mut crate::EchoFn) };
        }
        _ => {
            // Unknown / non-owned kinds (sockets, tasks): do not free.
            // Unmark dead so we don't pretend we freed.
            return;
        }
    }
    crate::note_heap_free(handle);
    let _ = HEAP_MAGIC; // keep magic used
}

/// Test helper: number of open scopes.
#[cfg(test)]
pub fn test_open_scope_count() -> usize {
    STATE.with(|c| c.borrow().stack.len())
}

/// Test helper: whether handle is owned.
#[cfg(test)]
pub fn test_is_owned(handle: i64) -> bool {
    STATE.with(|c| c.borrow().owner.contains_key(&handle))
}

/// Test helper: clear all scope state (tests only).
#[cfg(test)]
pub fn test_reset() {
    STATE.with(|c| {
        let mut st = c.borrow_mut();
        // Free remaining without process exit.
        let owned: Vec<i64> = st.owner.keys().copied().collect();
        for h in owned {
            if st.dead.insert(h) {
                physical_free(h);
            }
        }
        st.stack.clear();
        st.owner.clear();
        st.dead.clear();
        let def = std::mem::take(&mut st.deferred);
        for h in def {
            physical_free(h);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::echo_runtime_list_new;

    #[test]
    fn enter_register_exit_releases() {
        test_reset();
        echo_runtime_scope_enter(1);
        let h = echo_runtime_list_new();
        echo_runtime_scope_register(h);
        assert!(test_is_owned(h));
        echo_runtime_scope_exit(1);
        assert!(!test_is_owned(h));
        assert_eq!(test_open_scope_count(), 0);
        test_reset();
    }

    #[test]
    fn promote_moves_owner() {
        test_reset();
        echo_runtime_scope_enter(1); // outer
        echo_runtime_scope_enter(2); // inner
        let h = echo_runtime_list_new();
        echo_runtime_scope_register(h);
        echo_runtime_scope_promote(h, 1);
        echo_runtime_scope_exit(2); // must not free h
        assert!(test_is_owned(h));
        echo_runtime_scope_exit(1);
        assert!(!test_is_owned(h));
        test_reset();
    }

    #[test]
    fn reentrant_root_scope_ids() {
        test_reset();
        echo_runtime_scope_enter(0);
        let outer = echo_runtime_list_new();
        echo_runtime_scope_register(outer);
        echo_runtime_scope_enter(0); // nested call frame, same compile-time id
        let inner = echo_runtime_list_new();
        echo_runtime_scope_register(inner);
        echo_runtime_scope_exit(0); // frees inner only
        assert!(!test_is_owned(inner));
        assert!(test_is_owned(outer));
        echo_runtime_scope_exit(0);
        assert!(!test_is_owned(outer));
        test_reset();
    }

    #[test]
    fn double_release_is_idempotent() {
        test_reset();
        echo_runtime_scope_enter(1);
        let h = echo_runtime_list_new();
        echo_runtime_scope_register(h);
        echo_runtime_scope_release(h);
        echo_runtime_scope_release(h); // no crash
        echo_runtime_scope_exit(1);
        test_reset();
    }
}
