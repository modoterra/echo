use crate::{MirExpr, MirFunctionCall};
use echo_ast::Stmt;

#[derive(Debug, Clone, PartialEq)]
pub enum MirStmt {
    Echo {
        source: Stmt,
        exprs: Vec<MirExpr>,
    },
    FunctionCall {
        source: Stmt,
        call: MirFunctionCall,
    },
    DynamicFunctionCall {
        source: Stmt,
        name: String,
        args: Vec<MirExpr>,
    },
    Assign {
        source: Stmt,
        name: String,
        value: MirExpr,
    },
    Let {
        source: Stmt,
        name: String,
        value: MirExpr,
    },
    Return {
        source: Stmt,
        value: MirExpr,
    },
    Expr {
        source: Stmt,
        expr: MirExpr,
    },
    Loop {
        source: Stmt,
        body: Vec<MirStmt>,
    },
    If {
        source: Stmt,
        condition: MirExpr,
        body: Vec<MirStmt>,
    },
    Break {
        source: Stmt,
        value: Option<MirExpr>,
    },
    Append {
        source: Stmt,
        target: String,
        value: MirExpr,
    },
    AssignRef {
        source: Stmt,
        name: String,
        target: String,
    },
    Yield {
        source: Stmt,
        value: MirExpr,
    },
    Noop {
        source: Stmt,
    },
}

impl MirStmt {
    pub fn syntax(&self) -> &Stmt {
        match self {
            Self::Echo { source, .. }
            | Self::FunctionCall { source, .. }
            | Self::DynamicFunctionCall { source, .. }
            | Self::Assign { source, .. }
            | Self::Let { source, .. }
            | Self::Return { source, .. }
            | Self::Expr { source, .. }
            | Self::Loop { source, .. }
            | Self::If { source, .. }
            | Self::Break { source, .. }
            | Self::Append { source, .. }
            | Self::AssignRef { source, .. }
            | Self::Yield { source, .. }
            | Self::Noop { source } => source,
        }
    }
}
