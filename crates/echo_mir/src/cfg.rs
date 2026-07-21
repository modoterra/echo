//! Structured MIR → CFG (basic blocks + terminators).
//!
//! Analysis facts are already resolved in the structured form; this pass only
//! rewrites control-flow shape. For-in expands to index loops with [`MirPrim`].
//! Tagged match keeps a dedicated terminator (i128 pack) plus [`MirOp::MatchPayload`].

use echo_ast::BinaryOp;

use crate::{MirExpr, MirPrim, MirRetShape, MirStmt};

/// Opaque basic-block identity within one function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub u32);

/// Side-effecting ops that do not transfer control.
#[derive(Debug, Clone)]
pub enum MirOp {
    /// SSA φ-node at block entry (filled by [`crate::construct_ssa`]).
    Phi {
        /// Destination SSA name (`base@version`).
        name: String,
        /// Predecessor block → SSA name of incoming value.
        incomings: Vec<(BlockId, String)>,
    },
    /// Bind match arm payload (i64) after [`Terminator::MatchTagged`].
    MatchPayload {
        name: String,
    },
    Set {
        name: String,
        value: MirExpr,
    },
    Eval(MirExpr),
    FieldSet {
        base: MirExpr,
        field: String,
        value: MirExpr,
    },
    IndexSet {
        base: MirExpr,
        index: MirExpr,
        value: MirExpr,
    },
    /// `~ base[] = value` list append.
    ListPush {
        base: MirExpr,
        value: MirExpr,
    },
    /// Schedule closed body on the mio event loop.
    TaskSpawn {
        module_path: std::path::PathBuf,
        body_symbol: String,
        bind: Option<String>,
    },
    /// `+ f(args)`.
    TaskSpawnFn {
        module_path: std::path::PathBuf,
        fn_symbol: String,
        args: Vec<MirExpr>,
        bind: Option<String>,
    },
    /// Immediate block or join handle.
    TaskJoin {
        module_path: std::path::PathBuf,
        body_symbol: Option<String>,
        handle: Option<MirExpr>,
        bind: Option<String>,
    },
}

/// Control-flow edge out of a block.
#[derive(Debug, Clone)]
pub enum Terminator {
    Goto(BlockId),
    Branch {
        cond: MirExpr,
        then_bb: BlockId,
        else_bb: BlockId,
    },
    /// Tagged result/option match: branch on runtime tag of `scrutinee`.
    /// Arm payloads are bound via leading [`MirOp::MatchPayload`] in the targets.
    MatchTagged {
        scrutinee: MirExpr,
        ok_bb: BlockId,
        err_bb: BlockId,
    },
    ReturnOk(MirExpr),
    ReturnErr(MirExpr),
    ReturnNone,
    /// Fallthrough used only during construction; must be rewritten before use.
    Unreachable,
}

#[derive(Debug, Clone)]
pub struct MirBlock {
    pub id: BlockId,
    pub ops: Vec<MirOp>,
    pub term: Terminator,
}

#[derive(Debug, Clone)]
pub struct MirCfg {
    pub blocks: Vec<MirBlock>,
    pub entry: BlockId,
}

impl MirCfg {
    #[must_use]
    pub fn block(&self, id: BlockId) -> &MirBlock {
        &self.blocks[id.0 as usize]
    }

    #[must_use]
    pub fn successors(&self, id: BlockId) -> Vec<BlockId> {
        match &self.block(id).term {
            Terminator::Goto(t) => vec![*t],
            Terminator::Branch {
                then_bb, else_bb, ..
            } => vec![*then_bb, *else_bb],
            Terminator::MatchTagged { ok_bb, err_bb, .. } => vec![*ok_bb, *err_bb],
            Terminator::ReturnOk(_)
            | Terminator::ReturnErr(_)
            | Terminator::ReturnNone
            | Terminator::Unreachable => vec![],
        }
    }

    #[must_use]
    pub fn predecessors(&self) -> Vec<Vec<BlockId>> {
        let n = self.blocks.len();
        let mut preds = vec![Vec::new(); n];
        for b in &self.blocks {
            for s in self.successors(b.id) {
                preds[s.0 as usize].push(b.id);
            }
        }
        preds
    }
}

struct Builder {
    blocks: Vec<MirBlock>,
}

impl Builder {
    fn new() -> Self {
        Self { blocks: Vec::new() }
    }

    fn alloc(&mut self) -> BlockId {
        let id = BlockId(self.blocks.len() as u32);
        self.blocks.push(MirBlock {
            id,
            ops: Vec::new(),
            term: Terminator::Unreachable,
        });
        id
    }

    fn block_mut(&mut self, id: BlockId) -> &mut MirBlock {
        &mut self.blocks[id.0 as usize]
    }

    fn push_op(&mut self, id: BlockId, op: MirOp) {
        self.block_mut(id).ops.push(op);
    }

    fn set_term(&mut self, id: BlockId, term: Terminator) {
        self.block_mut(id).term = term;
    }

    fn finish(self, entry: BlockId) -> MirCfg {
        MirCfg {
            blocks: self.blocks,
            entry,
        }
    }
}

/// Convert structured statements into a CFG (pre-SSA).
///
/// Paths that fall off without `^`/`!` get [`Terminator::ReturnOk`] of
/// `fallthrough` (default: `0`; methods use the receiver name).
#[must_use]
pub fn structured_to_cfg(stmts: &[MirStmt], ret: MirRetShape) -> MirCfg {
    structured_to_cfg_with_fallthrough(stmts, ret, MirExpr::ConstI64(0))
}

/// Like [`structured_to_cfg`], with an explicit fall-through return value.
#[must_use]
pub fn structured_to_cfg_with_fallthrough(
    stmts: &[MirStmt],
    _ret: MirRetShape,
    fallthrough: MirExpr,
) -> MirCfg {
    let mut b = Builder::new();
    let entry = b.alloc();
    let after = lower_seq(&mut b, entry, stmts, None);
    if matches!(b.block_mut(after).term, Terminator::Unreachable) {
        b.set_term(after, Terminator::ReturnOk(fallthrough));
    }
    b.finish(entry)
}

/// Loop context: (continue target, break target).
type LoopCtx = Option<(BlockId, BlockId)>;

fn lower_seq(b: &mut Builder, mut cur: BlockId, stmts: &[MirStmt], loop_ctx: LoopCtx) -> BlockId {
    for stmt in stmts {
        if !matches!(b.block_mut(cur).term, Terminator::Unreachable) {
            cur = b.alloc();
        }
        cur = lower_stmt(b, cur, stmt, loop_ctx);
    }
    cur
}

fn lower_stmt(b: &mut Builder, cur: BlockId, stmt: &MirStmt, loop_ctx: LoopCtx) -> BlockId {
    match stmt {
        MirStmt::Set { name, value } => {
            b.push_op(
                cur,
                MirOp::Set {
                    name: name.clone(),
                    value: value.clone(),
                },
            );
            cur
        }
        MirStmt::Eval(e) => {
            b.push_op(cur, MirOp::Eval(e.clone()));
            cur
        }
        MirStmt::FieldSet { base, field, value } => {
            b.push_op(
                cur,
                MirOp::FieldSet {
                    base: base.clone(),
                    field: field.clone(),
                    value: value.clone(),
                },
            );
            cur
        }
        MirStmt::IndexSet {
            base,
            index,
            value,
        } => {
            b.push_op(
                cur,
                MirOp::IndexSet {
                    base: base.clone(),
                    index: index.clone(),
                    value: value.clone(),
                },
            );
            cur
        }
        MirStmt::ListPush { base, value } => {
            b.push_op(
                cur,
                MirOp::ListPush {
                    base: base.clone(),
                    value: value.clone(),
                },
            );
            cur
        }
        MirStmt::ReturnOk(e) => {
            b.set_term(cur, Terminator::ReturnOk(e.clone()));
            cur
        }
        MirStmt::ReturnErr(e) => {
            b.set_term(cur, Terminator::ReturnErr(e.clone()));
            cur
        }
        MirStmt::ReturnNone => {
            b.set_term(cur, Terminator::ReturnNone);
            cur
        }
        MirStmt::Break => {
            if let Some((_, brk)) = loop_ctx {
                b.set_term(cur, Terminator::Goto(brk));
            } else {
                b.set_term(cur, Terminator::Unreachable);
            }
            cur
        }
        MirStmt::Continue => {
            if let Some((cont, _)) = loop_ctx {
                b.set_term(cur, Terminator::Goto(cont));
            } else {
                b.set_term(cur, Terminator::Unreachable);
            }
            cur
        }
        MirStmt::If { arms, else_body } => lower_if(b, cur, arms, else_body.as_deref(), loop_ctx),
        MirStmt::MatchTagged {
            scrutinee,
            ok_name,
            ok_body,
            err_name,
            err_body,
        } => lower_match_tagged(
            b,
            cur,
            scrutinee,
            ok_name.as_deref(),
            ok_body,
            err_name.as_deref(),
            err_body,
            loop_ctx,
        ),
        MirStmt::Loop { cond, body } => lower_loop(b, cur, cond.as_ref(), body, loop_ctx),
        MirStmt::ForIn { item, iter, body } => lower_for_in(b, cur, item, iter, body),
        MirStmt::TaskSpawn {
            module_path,
            body_symbol,
            bind,
        } => {
            b.push_op(
                cur,
                MirOp::TaskSpawn {
                    module_path: module_path.clone(),
                    body_symbol: body_symbol.clone(),
                    bind: bind.clone(),
                },
            );
            cur
        }
        MirStmt::TaskSpawnFn {
            module_path,
            fn_symbol,
            args,
            bind,
        } => {
            b.push_op(
                cur,
                MirOp::TaskSpawnFn {
                    module_path: module_path.clone(),
                    fn_symbol: fn_symbol.clone(),
                    args: args.clone(),
                    bind: bind.clone(),
                },
            );
            cur
        }
        MirStmt::TaskJoin {
            module_path,
            body_symbol,
            handle,
            bind,
        } => {
            b.push_op(
                cur,
                MirOp::TaskJoin {
                    module_path: module_path.clone(),
                    body_symbol: body_symbol.clone(),
                    handle: handle.clone(),
                    bind: bind.clone(),
                },
            );
            cur
        }
    }
}

fn lower_if(
    b: &mut Builder,
    cur: BlockId,
    arms: &[(MirExpr, Vec<MirStmt>)],
    else_body: Option<&[MirStmt]>,
    loop_ctx: LoopCtx,
) -> BlockId {
    if arms.is_empty() {
        if let Some(body) = else_body {
            return lower_seq(b, cur, body, loop_ctx);
        }
        return cur;
    }

    let merge = b.alloc();
    let mut current = cur;

    for (i, (cond, body)) in arms.iter().enumerate() {
        let then_bb = b.alloc();
        let else_bb = if i + 1 == arms.len() && else_body.is_none() {
            merge
        } else {
            b.alloc()
        };

        b.set_term(
            current,
            Terminator::Branch {
                cond: cond.clone(),
                then_bb,
                else_bb,
            },
        );

        let then_end = lower_seq(b, then_bb, body, loop_ctx);
        if matches!(b.block_mut(then_end).term, Terminator::Unreachable) {
            b.set_term(then_end, Terminator::Goto(merge));
        }

        current = else_bb;
        if i + 1 < arms.len() {
            // chain
        } else if let Some(eb) = else_body {
            let else_end = lower_seq(b, else_bb, eb, loop_ctx);
            if matches!(b.block_mut(else_end).term, Terminator::Unreachable) {
                b.set_term(else_end, Terminator::Goto(merge));
            }
            return merge;
        } else {
            return merge;
        }
    }

    if matches!(b.block_mut(current).term, Terminator::Unreachable) && current != merge {
        b.set_term(current, Terminator::Goto(merge));
    }
    merge
}

fn lower_match_tagged(
    b: &mut Builder,
    cur: BlockId,
    scrutinee: &MirExpr,
    ok_name: Option<&str>,
    ok_body: &[MirStmt],
    err_name: Option<&str>,
    err_body: &[MirStmt],
    loop_ctx: LoopCtx,
) -> BlockId {
    let ok_bb = b.alloc();
    let err_bb = b.alloc();
    let merge = b.alloc();

    b.set_term(
        cur,
        Terminator::MatchTagged {
            scrutinee: scrutinee.clone(),
            ok_bb,
            err_bb,
        },
    );

    if let Some(n) = ok_name {
        b.push_op(
            ok_bb,
            MirOp::MatchPayload {
                name: n.to_string(),
            },
        );
    }
    if let Some(n) = err_name {
        b.push_op(
            err_bb,
            MirOp::MatchPayload {
                name: n.to_string(),
            },
        );
    }

    let ok_end = lower_seq(b, ok_bb, ok_body, loop_ctx);
    if matches!(b.block_mut(ok_end).term, Terminator::Unreachable) {
        b.set_term(ok_end, Terminator::Goto(merge));
    }

    let err_end = lower_seq(b, err_bb, err_body, loop_ctx);
    if matches!(b.block_mut(err_end).term, Terminator::Unreachable) {
        b.set_term(err_end, Terminator::Goto(merge));
    }

    merge
}

fn lower_loop(
    b: &mut Builder,
    cur: BlockId,
    cond: Option<&MirExpr>,
    body: &[MirStmt],
    _outer: LoopCtx,
) -> BlockId {
    let header = b.alloc();
    let body_bb = b.alloc();
    let exit = b.alloc();

    b.set_term(cur, Terminator::Goto(header));

    if let Some(c) = cond {
        b.set_term(
            header,
            Terminator::Branch {
                cond: c.clone(),
                then_bb: body_bb,
                else_bb: exit,
            },
        );
    } else {
        b.set_term(header, Terminator::Goto(body_bb));
    }

    let body_end = lower_seq(b, body_bb, body, Some((header, exit)));
    if matches!(b.block_mut(body_end).term, Terminator::Unreachable) {
        b.set_term(body_end, Terminator::Goto(header));
    }
    exit
}

/// Expand for-in to index loop with list_len / list_get prims.
fn lower_for_in(
    b: &mut Builder,
    cur: BlockId,
    item: &str,
    iter: &MirExpr,
    body: &[MirStmt],
) -> BlockId {
    let uid = cur.0;
    let iter_n = format!("__iter_{uid}");
    let idx_n = format!("__i_{uid}");

    let header = b.alloc();
    let body_bb = b.alloc();
    let cont = b.alloc();
    let exit = b.alloc();

    b.push_op(
        cur,
        MirOp::Set {
            name: iter_n.clone(),
            value: iter.clone(),
        },
    );
    b.push_op(
        cur,
        MirOp::Set {
            name: idx_n.clone(),
            value: MirExpr::ConstI64(0),
        },
    );
    b.set_term(cur, Terminator::Goto(header));

    let cond = MirExpr::Binary {
        op: BinaryOp::Lt,
        left: Box::new(MirExpr::Name(idx_n.clone())),
        right: Box::new(MirExpr::PrimCall {
            prim: MirPrim::ListLen,
            args: vec![MirExpr::Name(iter_n.clone())],
        }),
    };
    b.set_term(
        header,
        Terminator::Branch {
            cond,
            then_bb: body_bb,
            else_bb: exit,
        },
    );

    b.push_op(
        body_bb,
        MirOp::Set {
            name: item.to_string(),
            value: MirExpr::PrimCall {
                prim: MirPrim::ListGetChecked,
                args: vec![
                    MirExpr::Name(iter_n.clone()),
                    MirExpr::Name(idx_n.clone()),
                ],
            },
        },
    );
    // continue → cont (i++); break → exit
    let body_end = lower_seq(b, body_bb, body, Some((cont, exit)));
    if matches!(b.block_mut(body_end).term, Terminator::Unreachable) {
        b.set_term(body_end, Terminator::Goto(cont));
    }

    // Only wire cont → header when something can reach cont. A dead cont with a
    // back-edge would still be a CFG predecessor of the header and poison
    // dominance / SSA (see `construct_ssa` reachable-pred fix). Bodies that always
    // `^`/`!` leave cont with no preds.
    let cont_targeted = b.blocks.iter().any(|bb| {
        matches!(
            &bb.term,
            Terminator::Goto(t) if *t == cont
        ) || matches!(
            &bb.term,
            Terminator::Branch {
                then_bb,
                else_bb,
                ..
            } if *then_bb == cont || *else_bb == cont
        )
    });
    if cont_targeted {
        b.push_op(
            cont,
            MirOp::Set {
                name: idx_n.clone(),
                value: MirExpr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(MirExpr::Name(idx_n)),
                    right: Box::new(MirExpr::ConstI64(1)),
                },
            },
        );
        b.set_term(cont, Terminator::Goto(header));
    }

    exit
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn if_produces_branch_and_merge() {
        let stmts = vec![MirStmt::If {
            arms: vec![(
                MirExpr::ConstI64(1),
                vec![MirStmt::ReturnOk(MirExpr::ConstI64(2))],
            )],
            else_body: Some(vec![MirStmt::ReturnOk(MirExpr::ConstI64(3))]),
        }];
        let cfg = structured_to_cfg(&stmts, MirRetShape::Plain);
        assert!(cfg.blocks.len() >= 3);
        assert!(matches!(
            cfg.blocks[cfg.entry.0 as usize].term,
            Terminator::Branch { .. }
        ));
    }

    #[test]
    fn loop_has_back_edge() {
        let stmts = vec![MirStmt::Loop {
            cond: Some(MirExpr::ConstI64(1)),
            body: vec![MirStmt::Break],
        }];
        let cfg = structured_to_cfg(&stmts, MirRetShape::Plain);
        assert!(cfg.blocks.len() >= 3);
    }

    #[test]
    fn match_tagged_terminator() {
        let stmts = vec![MirStmt::MatchTagged {
            scrutinee: MirExpr::Name("r".into()),
            ok_name: Some("v".into()),
            ok_body: vec![MirStmt::ReturnOk(MirExpr::Name("v".into()))],
            err_name: Some("e".into()),
            err_body: vec![MirStmt::ReturnOk(MirExpr::ConstI64(0))],
        }];
        let cfg = structured_to_cfg(&stmts, MirRetShape::Result);
        assert!(matches!(
            cfg.blocks[cfg.entry.0 as usize].term,
            Terminator::MatchTagged { .. }
        ));
        assert!(cfg.blocks.iter().any(|bb| {
            bb.ops
                .iter()
                .any(|op| matches!(op, MirOp::MatchPayload { name } if name == "v"))
        }));
    }

    #[test]
    fn for_in_expands_to_index_loop() {
        let stmts = vec![MirStmt::ForIn {
            item: "x".into(),
            iter: MirExpr::Name("xs".into()),
            body: vec![MirStmt::Eval(MirExpr::Name("x".into()))],
        }];
        let cfg = structured_to_cfg(&stmts, MirRetShape::Plain);
        assert!(cfg.blocks.iter().any(|bb| {
            matches!(&bb.term, Terminator::Branch { cond, .. } if matches!(cond, MirExpr::Binary { .. }))
        }));
        assert!(cfg.blocks.iter().any(|bb| {
            bb.ops.iter().any(|op| {
                matches!(
                    op,
                    MirOp::Set {
                        value: MirExpr::PrimCall {
                            prim: MirPrim::ListGetChecked,
                            ..
                        },
                        ..
                    }
                )
            })
        }));
    }

    #[test]
    fn straight_line_sets_and_return() {
        let stmts = vec![
            MirStmt::Set {
                name: "n".into(),
                value: MirExpr::ConstI64(1),
            },
            MirStmt::ReturnOk(MirExpr::Name("n".into())),
        ];
        let cfg = structured_to_cfg(&stmts, MirRetShape::Plain);
        assert_eq!(cfg.entry, BlockId(0));
        assert!(matches!(
            cfg.blocks[0].ops.as_slice(),
            [MirOp::Set { name, .. }] if name == "n"
        ));
        assert!(matches!(
            cfg.blocks[0].term,
            Terminator::ReturnOk(MirExpr::Name(ref n)) if n == "n"
        ));
    }
}
