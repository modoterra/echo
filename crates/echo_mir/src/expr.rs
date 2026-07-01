use crate::MirStmt;
use echo_ast::{BinaryOp, Expr, UnaryOp};
use echo_source::Span;
use std::ops::Deref;

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
    Constant {
        source: Expr,
        name: String,
    },
    ReceiverConst {
        source: Expr,
        kind: echo_ast::ReceiverConst,
    },
    StaticPropertyFetch {
        source: Expr,
        class_name: echo_ast::QualifiedName,
        property: String,
    },
    StaticPropertyAssign {
        source: Expr,
        class_name: echo_ast::QualifiedName,
        property: String,
        value: Box<MirExpr>,
    },
    ClassConstantFetch {
        source: Expr,
        class_name: echo_ast::QualifiedName,
        constant: String,
    },
    FunctionCall {
        source: Expr,
        call: MirFunctionCall,
    },
    Print {
        source: Expr,
        value: Box<MirExpr>,
    },
    DynamicFunctionCall {
        source: Expr,
        name: String,
        args: Vec<MirCallArg>,
    },
    DynamicCall {
        source: Expr,
        callee: Box<MirExpr>,
        args: Vec<MirCallArg>,
    },
    MethodCall {
        source: Expr,
        object: Box<MirExpr>,
        method: String,
        method_span: Span,
        args: Vec<MirCallArg>,
    },
    StaticCall {
        source: Expr,
        class_name: echo_ast::QualifiedName,
        method: String,
        args: Vec<MirCallArg>,
    },
    New {
        source: Expr,
        target: MirNewTarget,
        args: Vec<MirCallArg>,
    },
    Closure {
        source: Expr,
        params: Vec<echo_ast::TypedParam>,
        captures: Vec<String>,
        return_type: Option<String>,
        body: Vec<MirStmt>,
    },
    ArrowFunction {
        source: Expr,
        params: Vec<echo_ast::TypedParam>,
        return_type: Option<String>,
        body: Box<MirExpr>,
    },
    Assign {
        source: Expr,
        name: String,
        value: Box<MirExpr>,
    },
    MagicDir {
        source: Expr,
    },
    Include {
        source: Expr,
        kind: echo_ast::IncludeKind,
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
    PhpCloneWith {
        source: Expr,
        object: Box<MirExpr>,
        updates: Box<MirExpr>,
    },
    Cast {
        source: Expr,
        ty: String,
        expr: Box<MirExpr>,
    },
    Binary {
        source: Expr,
        left: Box<MirExpr>,
        op: BinaryOp,
        right: Box<MirExpr>,
    },
    Ternary {
        source: Expr,
        condition: Box<MirExpr>,
        if_true: Box<MirExpr>,
        if_false: Box<MirExpr>,
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
    TargetAssign {
        source: Expr,
        target: Box<MirExpr>,
        value: Box<MirExpr>,
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
pub enum MirNewTarget {
    Class(echo_ast::QualifiedName),
    Expr(Box<MirExpr>),
    AnonymousClass(echo_ast::AnonymousClassExpr),
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
    pub args: Vec<MirCallArg>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MirCallArg {
    pub name: Option<String>,
    pub value: MirExpr,
    pub span: Span,
}

impl Deref for MirCallArg {
    type Target = MirExpr;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
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
            | Self::Constant { source, .. }
            | Self::ReceiverConst { source, .. }
            | Self::StaticPropertyFetch { source, .. }
            | Self::StaticPropertyAssign { source, .. }
            | Self::ClassConstantFetch { source, .. }
            | Self::FunctionCall { source, .. }
            | Self::Print { source, .. }
            | Self::DynamicFunctionCall { source, .. }
            | Self::DynamicCall { source, .. }
            | Self::MethodCall { source, .. }
            | Self::StaticCall { source, .. }
            | Self::New { source, .. }
            | Self::Closure { source, .. }
            | Self::ArrowFunction { source, .. }
            | Self::Assign { source, .. }
            | Self::MagicDir { source }
            | Self::Include { source, .. }
            | Self::Defer { source, .. }
            | Self::Run { source, .. }
            | Self::Fork { source, .. }
            | Self::Spawn { source, .. }
            | Self::Join { source, .. }
            | Self::Loop { source, .. }
            | Self::Unary { source, .. }
            | Self::PhpCloneWith { source, .. }
            | Self::Cast { source, .. }
            | Self::Binary { source, .. }
            | Self::Ternary { source, .. }
            | Self::Field { source, .. }
            | Self::Index { source, .. }
            | Self::TargetAssign { source, .. }
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
            | Self::Constant { source, .. }
            | Self::ReceiverConst { source, .. }
            | Self::StaticPropertyFetch { source, .. }
            | Self::StaticPropertyAssign { source, .. }
            | Self::ClassConstantFetch { source, .. }
            | Self::FunctionCall { source, .. }
            | Self::Print { source, .. }
            | Self::DynamicFunctionCall { source, .. }
            | Self::DynamicCall { source, .. }
            | Self::MethodCall { source, .. }
            | Self::StaticCall { source, .. }
            | Self::New { source, .. }
            | Self::Closure { source, .. }
            | Self::ArrowFunction { source, .. }
            | Self::Assign { source, .. }
            | Self::MagicDir { source }
            | Self::Include { source, .. }
            | Self::Defer { source, .. }
            | Self::Run { source, .. }
            | Self::Fork { source, .. }
            | Self::Spawn { source, .. }
            | Self::Join { source, .. }
            | Self::Loop { source, .. }
            | Self::Unary { source, .. }
            | Self::PhpCloneWith { source, .. }
            | Self::Cast { source, .. }
            | Self::Binary { source, .. }
            | Self::Ternary { source, .. }
            | Self::Field { source, .. }
            | Self::Index { source, .. }
            | Self::TargetAssign { source, .. }
            | Self::Object { source, .. }
            | Self::List { source, .. }
            | Self::Array { source, .. } => source.clone(),
        }
    }
}
