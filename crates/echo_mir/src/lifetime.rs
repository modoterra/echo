//! Scope-owned memory lowering (ADR 0016) — slice 1.
//!
//! Injects explicit `ScopeEnter` / `ScopeExit` / `ScopeRegister` / `ScopePromote`
//! / `ScopeDisown` into structured MIR. Semantics will later own richer facts;
//! this pass is the first vertical that is **sound and conservative**:
//! promote outward when uncertain; never free early.

use crate::{MirExpr, MirStmt};

/// Function root scope id.
pub const ROOT_SCOPE: u32 = 0;

/// Rewrite a function body with scope ownership ops.
#[must_use]
pub fn inject_lifetime(body: Vec<MirStmt>) -> Vec<MirStmt> {
    let mut next_id = 1u32;
    let mut out = Vec::with_capacity(body.len() + 4);
    // `open` is the stack of scope ids from root to current (for break/return edges).
    let mut open = vec![ROOT_SCOPE];
    // Innermost loop body scope ids (for break/continue cleanup depth).
    let mut loop_scopes: Vec<u32> = Vec::new();
    out.push(MirStmt::ScopeEnter { id: ROOT_SCOPE });
    out.extend(rewrite_seq(
        &body,
        &mut open,
        &mut loop_scopes,
        &mut next_id,
    ));
    // Fall-through exit (returns insert their own exits).
    out.push(MirStmt::ScopeExit { id: ROOT_SCOPE });
    out
}

fn rewrite_seq(
    stmts: &[MirStmt],
    open: &mut Vec<u32>,
    loop_scopes: &mut Vec<u32>,
    next_id: &mut u32,
) -> Vec<MirStmt> {
    let mut out = Vec::new();
    for s in stmts {
        out.extend(rewrite_stmt(s, open, loop_scopes, next_id));
    }
    out
}

fn current_scope(open: &[u32]) -> u32 {
    *open.last().unwrap_or(&ROOT_SCOPE)
}

fn rewrite_stmt(
    s: &MirStmt,
    open: &mut Vec<u32>,
    loop_scopes: &mut Vec<u32>,
    next_id: &mut u32,
) -> Vec<MirStmt> {
    let current = current_scope(open);
    match s {
        MirStmt::Set { name, value } => {
            let mut v = Vec::new();
            let (value2, extra) = rewrite_expr_alloc(value, current);
            v.extend(extra);
            // Escaping assignment into a binding may outlive this scope.
            // Conservative: promote managed non-fresh values outward to root.
            if expr_is_managed(&value2) && !expr_is_fresh_alloc(&value2) && current != ROOT_SCOPE {
                v.push(MirStmt::ScopePromote {
                    value: value2.clone(),
                    target: ROOT_SCOPE,
                });
            }
            v.push(MirStmt::Set {
                name: name.clone(),
                value: value2.clone(),
            });
            // Register only fresh allocations — aliases must not double-register.
            if expr_is_fresh_alloc(&value2) {
                v.push(MirStmt::ScopeRegister {
                    value: MirExpr::Name(name.clone()),
                });
            }
            v
        }
        MirStmt::FieldSet { base, field, value } => {
            let mut v = Vec::new();
            // Storing into a field: value must outlive current if base is outer.
            // Conservative: promote managed value to root.
            if expr_is_managed(value) && current != ROOT_SCOPE {
                v.push(MirStmt::ScopePromote {
                    value: value.clone(),
                    target: ROOT_SCOPE,
                });
            }
            v.push(MirStmt::FieldSet {
                base: base.clone(),
                field: field.clone(),
                value: value.clone(),
            });
            v
        }
        MirStmt::ListPush { base, value } => {
            let mut v = Vec::new();
            if expr_is_managed(value) && current != ROOT_SCOPE {
                v.push(MirStmt::ScopePromote {
                    value: value.clone(),
                    target: ROOT_SCOPE,
                });
            }
            v.push(MirStmt::ListPush {
                base: base.clone(),
                value: value.clone(),
            });
            v
        }
        MirStmt::IndexSet {
            base,
            index,
            value,
        } => {
            let mut v = Vec::new();
            if expr_is_managed(value) && current != ROOT_SCOPE {
                v.push(MirStmt::ScopePromote {
                    value: value.clone(),
                    target: ROOT_SCOPE,
                });
            }
            v.push(MirStmt::IndexSet {
                base: base.clone(),
                index: index.clone(),
                value: value.clone(),
            });
            v
        }
        MirStmt::If { arms, else_body } => {
            let mut new_arms = Vec::new();
            for (cond, body) in arms {
                let sid = *next_id;
                *next_id += 1;
                open.push(sid);
                let mut b = Vec::new();
                b.push(MirStmt::ScopeEnter { id: sid });
                b.extend(rewrite_seq(body, open, loop_scopes, next_id));
                b.push(MirStmt::ScopeExit { id: sid });
                open.pop();
                new_arms.push((cond.clone(), b));
            }
            let else_body = else_body.as_ref().map(|body| {
                let sid = *next_id;
                *next_id += 1;
                open.push(sid);
                let mut b = Vec::new();
                b.push(MirStmt::ScopeEnter { id: sid });
                b.extend(rewrite_seq(body, open, loop_scopes, next_id));
                b.push(MirStmt::ScopeExit { id: sid });
                open.pop();
                b
            });
            vec![MirStmt::If {
                arms: new_arms,
                else_body,
            }]
        }
        MirStmt::Loop { cond, body } => {
            let sid = *next_id;
            *next_id += 1;
            open.push(sid);
            loop_scopes.push(sid);
            let mut b = Vec::new();
            b.push(MirStmt::ScopeEnter { id: sid });
            b.extend(rewrite_seq(body, open, loop_scopes, next_id));
            b.push(MirStmt::ScopeExit { id: sid });
            loop_scopes.pop();
            open.pop();
            vec![MirStmt::Loop {
                cond: cond.clone(),
                body: b,
            }]
        }
        MirStmt::ForIn { item, iter, body } => {
            let sid = *next_id;
            *next_id += 1;
            open.push(sid);
            loop_scopes.push(sid);
            let mut b = Vec::new();
            b.push(MirStmt::ScopeEnter { id: sid });
            b.extend(rewrite_seq(body, open, loop_scopes, next_id));
            b.push(MirStmt::ScopeExit { id: sid });
            loop_scopes.pop();
            open.pop();
            vec![MirStmt::ForIn {
                item: item.clone(),
                iter: iter.clone(),
                body: b,
            }]
        }
        MirStmt::MatchTagged {
            scrutinee,
            ok_name,
            ok_body,
            err_name,
            err_body,
        } => {
            let ok_sid = *next_id;
            *next_id += 1;
            let err_sid = *next_id;
            *next_id += 1;
            open.push(ok_sid);
            let mut ok_b = Vec::new();
            ok_b.push(MirStmt::ScopeEnter { id: ok_sid });
            ok_b.extend(rewrite_seq(ok_body, open, loop_scopes, next_id));
            ok_b.push(MirStmt::ScopeExit { id: ok_sid });
            open.pop();
            open.push(err_sid);
            let mut err_b = Vec::new();
            err_b.push(MirStmt::ScopeEnter { id: err_sid });
            err_b.extend(rewrite_seq(err_body, open, loop_scopes, next_id));
            err_b.push(MirStmt::ScopeExit { id: err_sid });
            open.pop();
            vec![MirStmt::MatchTagged {
                scrutinee: scrutinee.clone(),
                ok_name: ok_name.clone(),
                ok_body: ok_b,
                err_name: err_name.clone(),
                err_body: err_b,
            }]
        }
        MirStmt::ReturnOk(e) => {
            exit_then_return(open, Some(ReturnKind::Ok(e.clone())), next_id)
        }
        MirStmt::ReturnErr(e) => {
            exit_then_return(open, Some(ReturnKind::Err(e.clone())), next_id)
        }
        MirStmt::ReturnNone => exit_then_return(open, Some(ReturnKind::None), next_id),
        // Break/continue leave the innermost loop body and any scopes nested in it.
        MirStmt::Break => {
            let mut v = exit_scopes_to_loop(open, loop_scopes);
            v.push(MirStmt::Break);
            v
        }
        MirStmt::Continue => {
            let mut v = exit_scopes_to_loop(open, loop_scopes);
            v.push(MirStmt::Continue);
            v
        }
        // Pass-through for other stmts (scope ops already explicit).
        other => vec![other.clone()],
    }
}

/// Exit open scopes from the top down through the innermost loop body scope.
fn exit_scopes_to_loop(open: &[u32], loop_scopes: &[u32]) -> Vec<MirStmt> {
    let mut v = Vec::new();
    let Some(&loop_sid) = loop_scopes.last() else {
        // Bare break outside loop — leave as-is for later diagnostics.
        return v;
    };
    for &id in open.iter().rev() {
        v.push(MirStmt::ScopeExit { id });
        if id == loop_sid {
            break;
        }
    }
    v
}

enum ReturnKind {
    Ok(MirExpr),
    Err(MirExpr),
    None,
}

/// Exit every open scope (innermost first) through root, disown return, return.
///
/// Managed returns that are **not** already a plain name are bound once to a
/// temp before disown/exit. Cloning a `Call` into both `ScopeDisown` and
/// `ReturnOk` used to **evaluate the call twice** (e.g. `^ .table.keys()` ran
/// `keys` twice), which is wasteful and can interact badly with ownership.
fn exit_then_return(
    open: &[u32],
    ret: Option<ReturnKind>,
    next_id: &mut u32,
) -> Vec<MirStmt> {
    let mut v = Vec::new();
    let ret = match ret {
        Some(ReturnKind::Ok(e)) if expr_is_managed(&e) => {
            let e = materialize_once(&mut v, e, next_id);
            v.push(MirStmt::ScopeDisown {
                value: e.clone(),
            });
            Some(ReturnKind::Ok(e))
        }
        Some(ReturnKind::Err(e)) if expr_is_managed(&e) => {
            let e = materialize_once(&mut v, e, next_id);
            v.push(MirStmt::ScopeDisown {
                value: e.clone(),
            });
            Some(ReturnKind::Err(e))
        }
        other => other,
    };
    for &id in open.iter().rev() {
        v.push(MirStmt::ScopeExit { id });
    }
    match ret {
        Some(ReturnKind::Ok(e)) => v.push(MirStmt::ReturnOk(e)),
        Some(ReturnKind::Err(e)) => v.push(MirStmt::ReturnErr(e)),
        Some(ReturnKind::None) => v.push(MirStmt::ReturnNone),
        None => {}
    }
    v
}

/// Bind `e` to a temp if it is not already a name; return a name expr.
fn materialize_once(v: &mut Vec<MirStmt>, e: MirExpr, next_id: &mut u32) -> MirExpr {
    if let MirExpr::Name(_) = &e {
        return e;
    }
    let n = format!("__ret_{}", *next_id);
    *next_id += 1;
    let is_fresh = expr_is_fresh_alloc(&e);
    v.push(MirStmt::Set {
        name: n.clone(),
        value: e,
    });
    if is_fresh {
        v.push(MirStmt::ScopeRegister {
            value: MirExpr::Name(n.clone()),
        });
    }
    MirExpr::Name(n)
}

fn rewrite_expr_alloc(e: &MirExpr, _current: u32) -> (MirExpr, Vec<MirStmt>) {
    // Allocations are registered after Set by name; no extra stmts here.
    (e.clone(), Vec::new())
}

/// True for expressions that create a **new** heap allocation (register once).
#[must_use]
pub fn expr_is_fresh_alloc(e: &MirExpr) -> bool {
    match e {
        MirExpr::ListLit(_)
        | MirExpr::StringLit { .. }
        | MirExpr::BytesLit { .. }
        | MirExpr::LocatorLit { .. }
        | MirExpr::StructLit { .. }
        | MirExpr::Range { .. }
        | MirExpr::FnValue { .. }
        | MirExpr::StringInterp { .. }
        | MirExpr::Call { .. } // callee disowns return; caller takes ownership
        | MirExpr::BoxValue { .. } => true,
        _ => false,
    }
}

/// Conservative: treat heap-bearing surface forms as managed (for promote/disown).
#[must_use]
pub fn expr_is_managed(e: &MirExpr) -> bool {
    if expr_is_fresh_alloc(e) {
        return true;
    }
    match e {
        MirExpr::Name(_) => true, // may be handle; runtime no-ops immediates
        MirExpr::PrimCall { prim, .. } => matches!(
            prim,
            crate::MirPrim::ListGetChecked // get may return managed
        ),
        MirExpr::FieldGet { .. } | MirExpr::Index { .. } => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MirRetShape, structured_to_cfg};

    #[test]
    fn break_exits_if_and_loop_scopes_in_cfg() {
        let body = vec![
            MirStmt::Loop {
                cond: None,
                body: vec![MirStmt::If {
                    arms: vec![(MirExpr::ConstI64(1), vec![MirStmt::Break])],
                    else_body: None,
                }],
            },
            MirStmt::ReturnOk(MirExpr::ConstI64(0)),
        ];
        let out = inject_lifetime(body);
        // Structured: break path should include ScopeExit before Break.
        let arm = match &out[1] {
            MirStmt::Loop { body, .. } => match &body[1] {
                MirStmt::If { arms, .. } => &arms[0].1,
                o => panic!("expected If, got {o:?}"),
            },
            o => panic!("expected Loop, got {o:?}"),
        };
        assert!(
            matches!(arm[0], MirStmt::ScopeEnter { id: 2 })
                || matches!(arm[0], MirStmt::ScopeEnter { .. }),
            "arm={arm:?}"
        );
        assert!(
            arm.iter().any(|s| matches!(s, MirStmt::ScopeExit { .. })),
            "arm missing ScopeExit before break: {arm:?}"
        );
        assert!(arm.iter().any(|s| matches!(s, MirStmt::Break)));

        let cfg = structured_to_cfg(&out, MirRetShape::Plain);
        let mut saw_exit_before_break = false;
        for b in &cfg.blocks {
            let has_exit = b
                .ops
                .iter()
                .any(|op| matches!(op, crate::MirOp::ScopeExit { .. }));
            let is_break = matches!(
                b.term,
                crate::Terminator::Goto(_) // break lowers to goto exit
            ) && b.ops.iter().any(|op| {
                matches!(op, crate::MirOp::ScopeEnter { id: 2 } | crate::MirOp::ScopeExit { id: 2 })
            });
            if is_break && has_exit {
                saw_exit_before_break = true;
            }
            // Dump-style assertion: any block that entered scope 2 must exit it
            // on the same path or leave via goto after exits.
        }
        // Stronger: find block with ScopeEnter 2; its ops must include ScopeExit 2
        // before terminator (break path).
        let enter2 = cfg.blocks.iter().find(|b| {
            b.ops
                .iter()
                .any(|op| matches!(op, crate::MirOp::ScopeEnter { id: 2 }))
        });
        let enter2 = enter2.expect("scope 2 enter");
        assert!(
            enter2
                .ops
                .iter()
                .any(|op| matches!(op, crate::MirOp::ScopeExit { id: 2 })),
            "enter-2 block missing exit-2: ops={:?} term={:?}",
            enter2.ops,
            enter2.term
        );
        let _ = saw_exit_before_break;

        // After SSA + simplify, ScopeExit must survive on the break path.
        let cfg2 = crate::construct_ssa(cfg, &[]);
        let (cfg2, reprs) = crate::analyze_reprs(cfg2, &[]);
        let (cfg2, _reprs) = crate::simplify_local(cfg2, reprs);
        let enter2 = cfg2
            .blocks
            .iter()
            .find(|b| {
                b.ops
                    .iter()
                    .any(|op| matches!(op, crate::MirOp::ScopeEnter { id: 2 }))
            })
            .expect("enter2 after ssa");
        assert!(
            enter2
                .ops
                .iter()
                .any(|op| matches!(op, crate::MirOp::ScopeExit { id: 2 })),
            "ScopeExit stripped after SSA/simplify: ops={:?} term={:?}",
            enter2.ops,
            enter2.term
        );
    }

    #[test]
    fn inject_nested_for_in_return_versions_indices() {
        use crate::{MirRetShape, MirStmt, MirExpr, inject_lifetime};
        use crate::cfg::structured_to_cfg;
        use crate::ssa::construct_ssa;
        // Nested for-in with return in inner body (hash_table is_empty shape).
        let body = inject_lifetime(vec![
            MirStmt::ForIn {
                item: "chain".into(),
                iter: MirExpr::Name("buckets".into()),
                body: vec![MirStmt::ForIn {
                    item: "e".into(),
                    iter: MirExpr::Name("chain".into()),
                    body: vec![MirStmt::ReturnOk(MirExpr::ConstBool(true))],
                }],
            },
            MirStmt::ReturnOk(MirExpr::ConstBool(false)),
        ]);
        let cfg = structured_to_cfg(&body, MirRetShape::Plain);
        let ssa = construct_ssa(cfg, &["buckets".into()]);
        // Any bare __i_* Name (no @) in ops or terms is a bug.
        fn walk_expr(e: &MirExpr, bad: &mut Vec<String>) {
            match e {
                MirExpr::Name(n) if n.starts_with("__i_") && !n.contains('@') => bad.push(n.clone()),
                MirExpr::Binary { left, right, .. } => {
                    walk_expr(left, bad);
                    walk_expr(right, bad);
                }
                MirExpr::PrimCall { args, .. } => {
                    for a in args {
                        walk_expr(a, bad);
                    }
                }
                MirExpr::Unary { expr, .. } | MirExpr::Cast { expr, .. } => walk_expr(expr, bad),
                _ => {}
            }
        }
        let mut bad = Vec::new();
        for b in &ssa.blocks {
            for op in &b.ops {
                match op {
                    crate::MirOp::Set { value, .. } => walk_expr(value, &mut bad),
                    crate::MirOp::ScopeRegister { value }
                    | crate::MirOp::ScopePromote { value, .. }
                    | crate::MirOp::ScopeDisown { value }
                    | crate::MirOp::ScopeRelease { value } => walk_expr(value, &mut bad),
                    crate::MirOp::Eval(e) => walk_expr(e, &mut bad),
                    _ => {}
                }
            }
            match &b.term {
                crate::Terminator::Branch { cond, .. } => walk_expr(cond, &mut bad),
                crate::Terminator::ReturnOk(e) | crate::Terminator::ReturnErr(e) => {
                    walk_expr(e, &mut bad)
                }
                _ => {}
            }
        }
        assert!(
            bad.is_empty(),
            "unversioned __i_* after SSA: {bad:?}\ncfg={ssa:#?}"
        );
    }

    #[test]
    fn inject_wraps_root_and_if() {
        let body = vec![
            MirStmt::Set {
                name: "xs".into(),
                value: MirExpr::ListLit(vec![]),
            },
            MirStmt::If {
                arms: vec![(
                    MirExpr::ConstI64(1),
                    vec![MirStmt::Set {
                        name: "ys".into(),
                        value: MirExpr::ListLit(vec![]),
                    }],
                )],
                else_body: None,
            },
            MirStmt::ReturnOk(MirExpr::Name("xs".into())),
        ];
        let out = inject_lifetime(body);
        assert!(matches!(out.first(), Some(MirStmt::ScopeEnter { id: 0 })));
        assert!(out.iter().any(|s| matches!(s, MirStmt::ScopeRegister { .. })));
        // Nested if arm scopes live inside MirStmt::If, not as top-level stmts.
        let has_arm_scope = out.iter().any(|s| match s {
            MirStmt::If { arms, .. } => arms
                .iter()
                .any(|(_, b)| b.iter().any(|x| matches!(x, MirStmt::ScopeEnter { id: 1 }))),
            _ => false,
        });
        assert!(has_arm_scope);
        // Return path exits scopes
        assert!(out.iter().any(|s| matches!(s, MirStmt::ScopeDisown { .. })));
        assert!(out.iter().any(|s| matches!(s, MirStmt::ScopeExit { id: 0 })));
    }

    #[test]
    fn return_call_materializes_once() {
        use crate::{CallTarget, MirExpr, MirRetShape, MirStmt};
        use std::path::PathBuf;
        let body = vec![MirStmt::ReturnOk(MirExpr::Call {
            target: CallTarget::Function {
                module_path: PathBuf::from("m"),
                name: "keys".into(),
            },
            args: vec![MirExpr::Name("t".into())],
            ret: MirRetShape::Plain,
        })];
        let out = inject_lifetime(body);
        let sets: Vec<_> = out
            .iter()
            .filter_map(|s| match s {
                MirStmt::Set { name, value } => Some((name.clone(), value.clone())),
                _ => None,
            })
            .collect();
        assert_eq!(sets.len(), 1, "expected one temp bind, got {out:?}");
        assert!(sets[0].0.starts_with("__ret_"), "{:?}", sets[0].0);
        let returns: Vec<_> = out
            .iter()
            .filter(|s| matches!(s, MirStmt::ReturnOk(_)))
            .collect();
        assert_eq!(returns.len(), 1);
        match &returns[0] {
            MirStmt::ReturnOk(MirExpr::Name(n)) => assert_eq!(n, &sets[0].0),
            other => panic!("return should be name, got {other:?}"),
        }
        fn count_calls(e: &MirExpr) -> usize {
            match e {
                MirExpr::Call { args, .. } => 1 + args.iter().map(count_calls).sum::<usize>(),
                MirExpr::FieldGet { base, .. } => count_calls(base),
                _ => 0,
            }
        }
        let mut n_calls = 0;
        for s in &out {
            match s {
                MirStmt::Set { value, .. } => n_calls += count_calls(value),
                MirStmt::ScopeDisown { value } => n_calls += count_calls(value),
                MirStmt::ReturnOk(e) | MirStmt::ReturnErr(e) => n_calls += count_calls(e),
                _ => {}
            }
        }
        assert_eq!(n_calls, 1, "call must appear once, out={out:?}");
    }
}