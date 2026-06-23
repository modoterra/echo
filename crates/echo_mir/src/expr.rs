use crate::MirStmt;
use echo_ast::{BinaryOp, Expr, UnaryOp};
use echo_source::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum MirExpr {
    Null {
        source: Expr,
    },
    Bool {
        source: Expr,
        value: bool,
    },
    String {
        source: Expr,
        value: String,
    },
    Number {
        source: Expr,
        value: String,
    },
    Variable {
        source: Expr,
        name: String,
    },
    FunctionCall {
        source: Expr,
        call: MirFunctionCall,
    },
    MethodCall {
        source: Expr,
        object: Box<MirExpr>,
        method: String,
        args: Vec<MirExpr>,
    },
    StaticCall {
        source: Expr,
        class_name: echo_ast::QualifiedName,
        method: String,
        args: Vec<MirExpr>,
    },
    Assign {
        source: Expr,
        name: String,
        value: Box<MirExpr>,
    },
    MagicDir {
        source: Expr,
    },
    Require {
        source: Expr,
        once: bool,
        path: Box<MirExpr>,
    },
    Defer {
        source: Expr,
        body: Vec<MirStmt>,
    },
    Run {
        source: Expr,
        expr: MirRunExpr,
    },
    Fork {
        source: Expr,
        expr: MirForkExpr,
    },
    Spawn {
        source: Expr,
        command: Box<MirExpr>,
    },
    Join {
        source: Expr,
        handle: Box<MirExpr>,
    },
    Loop {
        source: Expr,
        body: Vec<MirStmt>,
    },
    Unary {
        source: Expr,
        op: UnaryOp,
        expr: Box<MirExpr>,
    },
    Binary {
        source: Expr,
        left: Box<MirExpr>,
        op: BinaryOp,
        right: Box<MirExpr>,
    },
    Field {
        source: Expr,
        object: Box<MirExpr>,
        field: String,
    },
    Index {
        source: Expr,
        collection: Box<MirExpr>,
        index: Box<MirExpr>,
    },
    Object {
        source: Expr,
        name: String,
        fields: Vec<MirObjectField>,
    },
    List {
        source: Expr,
        values: Vec<MirExpr>,
    },
    Array {
        source: Expr,
        elements: Vec<MirArrayElement>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct MirObjectField {
    pub name: String,
    pub value: MirExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MirArrayElement {
    pub key: Option<MirExpr>,
    pub value: MirExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MirFunctionCall {
    pub name: String,
    pub args: Vec<MirExpr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MirRunExpr {
    Block { body: Vec<MirStmt> },
    Task { expr: Box<MirExpr> },
    Group { entries: Vec<Vec<MirStmt>> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum MirForkExpr {
    Block { body: Vec<MirStmt> },
    Task { expr: Box<MirExpr> },
}

impl MirExpr {
    pub fn syntax(&self) -> &Expr {
        match self {
            Self::Null { source }
            | Self::Bool { source, .. }
            | Self::String { source, .. }
            | Self::Number { source, .. }
            | Self::Variable { source, .. }
            | Self::FunctionCall { source, .. }
            | Self::MethodCall { source, .. }
            | Self::StaticCall { source, .. }
            | Self::Assign { source, .. }
            | Self::MagicDir { source }
            | Self::Require { source, .. }
            | Self::Defer { source, .. }
            | Self::Run { source, .. }
            | Self::Fork { source, .. }
            | Self::Spawn { source, .. }
            | Self::Join { source, .. }
            | Self::Loop { source, .. }
            | Self::Unary { source, .. }
            | Self::Binary { source, .. }
            | Self::Field { source, .. }
            | Self::Index { source, .. }
            | Self::Object { source, .. }
            | Self::List { source, .. }
            | Self::Array { source, .. } => source,
        }
    }

    pub fn to_syntax(&self) -> Expr {
        match self {
            Self::Null { source }
            | Self::Bool { source, .. }
            | Self::String { source, .. }
            | Self::Number { source, .. }
            | Self::Variable { source, .. }
            | Self::FunctionCall { source, .. }
            | Self::MethodCall { source, .. }
            | Self::StaticCall { source, .. }
            | Self::Assign { source, .. }
            | Self::MagicDir { source }
            | Self::Require { source, .. }
            | Self::Defer { source, .. }
            | Self::Run { source, .. }
            | Self::Fork { source, .. }
            | Self::Spawn { source, .. }
            | Self::Join { source, .. }
            | Self::Loop { source, .. }
            | Self::Unary { source, .. }
            | Self::Binary { source, .. }
            | Self::Field { source, .. }
            | Self::Index { source, .. }
            | Self::Object { source, .. }
            | Self::List { source, .. }
            | Self::Array { source, .. } => source.clone(),
        }
    }
}
