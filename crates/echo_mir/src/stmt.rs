use crate::{MirCallArg, MirExpr, MirFunctionCall};
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
        args: Vec<MirCallArg>,
    },
    Assign {
        source: Stmt,
        name: String,
        value: MirExpr,
    },
    CoalesceAssign {
        source: Stmt,
        name: String,
        value: MirExpr,
    },
    ListAssign {
        source: Stmt,
        targets: Vec<String>,
        value: MirExpr,
    },
    Let {
        source: Stmt,
        name: String,
        value: MirExpr,
    },
    Return {
        source: Stmt,
        value: Option<MirExpr>,
    },
    Throw {
        source: Stmt,
        value: MirExpr,
    },
    Goto {
        source: Stmt,
        label: String,
    },
    Label {
        source: Stmt,
        name: String,
    },
    PhpDeclare {
        source: Stmt,
        body: Vec<MirStmt>,
    },
    Expr {
        source: Stmt,
        expr: MirExpr,
    },
    Loop {
        source: Stmt,
        body: Vec<MirStmt>,
    },
    While {
        source: Stmt,
        condition: MirExpr,
        body: Vec<MirStmt>,
    },
    DoWhile {
        source: Stmt,
        body: Vec<MirStmt>,
        condition: MirExpr,
    },
    For {
        source: Stmt,
        init: Vec<MirExpr>,
        conditions: Vec<MirExpr>,
        increments: Vec<MirExpr>,
        body: Vec<MirStmt>,
    },
    Foreach {
        source: Stmt,
        iterable: MirExpr,
        key: Option<String>,
        value: String,
        body: Vec<MirStmt>,
    },
    Switch {
        source: Stmt,
        expr: MirExpr,
        cases: Vec<MirSwitchCase>,
    },
    If {
        source: Stmt,
        condition: MirExpr,
        body: Vec<MirStmt>,
        elseif_clauses: Vec<MirElseIfClause>,
        else_body: Vec<MirStmt>,
    },
    Try {
        source: Stmt,
        body: Vec<MirStmt>,
        catches: Vec<MirCatchClause>,
        finally_body: Vec<MirStmt>,
    },
    Break {
        source: Stmt,
        value: Option<MirExpr>,
    },
    Continue {
        source: Stmt,
        value: Option<MirExpr>,
    },
    Append {
        source: Stmt,
        target: MirExpr,
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

#[derive(Debug, Clone, PartialEq)]
pub struct MirElseIfClause {
    pub condition: MirExpr,
    pub body: Vec<MirStmt>,
    pub span: echo_source::Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MirSwitchCase {
    pub condition: Option<MirExpr>,
    pub body: Vec<MirStmt>,
    pub span: echo_source::Span,
}

impl MirStmt {
    pub fn syntax(&self) -> &Stmt {
        match self {
            Self::Echo { source, .. }
            | Self::FunctionCall { source, .. }
            | Self::DynamicFunctionCall { source, .. }
            | Self::Assign { source, .. }
            | Self::CoalesceAssign { source, .. }
            | Self::ListAssign { source, .. }
            | Self::Let { source, .. }
            | Self::Return { source, .. }
            | Self::Throw { source, .. }
            | Self::Goto { source, .. }
            | Self::Label { source, .. }
            | Self::PhpDeclare { source, .. }
            | Self::Expr { source, .. }
            | Self::Loop { source, .. }
            | Self::While { source, .. }
            | Self::DoWhile { source, .. }
            | Self::For { source, .. }
            | Self::Foreach { source, .. }
            | Self::Switch { source, .. }
            | Self::If { source, .. }
            | Self::Try { source, .. }
            | Self::Break { source, .. }
            | Self::Continue { source, .. }
            | Self::Append { source, .. }
            | Self::AssignRef { source, .. }
            | Self::Yield { source, .. }
            | Self::Noop { source } => source,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MirCatchClause {
    pub types: Vec<echo_ast::QualifiedName>,
    pub variable: Option<String>,
    pub body: Vec<MirStmt>,
    pub span: echo_source::Span,
}
