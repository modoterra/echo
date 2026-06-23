use crate::{Expr, QualifiedName};
use echo_source::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Echo(EchoStmt),
    FunctionCall(FunctionCallStmt),
    DynamicFunctionCall(DynamicFunctionCallStmt),
    FunctionDecl(FunctionDeclStmt),
    Assign(AssignStmt),
    Let(LetStmt),
    AssignRef(AssignRefStmt),
    Return(ReturnStmt),
    Yield(YieldStmt),
    Expr(ExprStmt),
    Namespace(NamespaceStmt),
    Use(UseStmt),
    Import(ImportStmt),
    ClassDecl(ClassDeclStmt),
    ExtendDecl(ExtendDeclStmt),
    TypeDecl(TypeDeclStmt),
    Loop(LoopStmt),
    If(IfStmt),
    Break(BreakStmt),
    Append(AppendStmt),
}

#[derive(Debug, Clone, PartialEq)]
pub struct EchoStmt {
    pub exprs: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCallStmt {
    pub name: String,
    pub args: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DynamicFunctionCallStmt {
    pub name: String,
    pub args: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclStmt {
    pub name: String,
    pub params: Vec<TypedParam>,
    pub return_type: Option<String>,
    pub is_intrinsic: bool,
    pub is_generator: bool,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssignStmt {
    pub name: String,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LetStmt {
    pub name: String,
    pub ty: Option<String>,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssignRefStmt {
    pub name: String,
    pub target: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStmt {
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct YieldStmt {
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprStmt {
    pub expr: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceStmt {
    pub source: NamespaceSource,
    pub name: QualifiedName,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamespaceSource {
    Php,
    Std,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UseStmt {
    pub name: QualifiedName,
    pub alias: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportStmt {
    pub source: ImportSource,
    pub name: QualifiedName,
    pub alias: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportSource {
    Std,
    File(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDeclStmt {
    pub name: String,
    pub parent: Option<QualifiedName>,
    pub members: Vec<ClassMember>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExtendDeclStmt {
    pub target: QualifiedName,
    pub members: Vec<ClassMember>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDeclStmt {
    pub name: String,
    pub fields: Vec<TypeField>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeField {
    pub name: String,
    pub ty: String,
    pub is_const: bool,
    pub is_optional: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoopStmt {
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStmt {
    pub condition: Expr,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BreakStmt {
    pub value: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AppendStmt {
    pub target: String,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClassMember {
    Method(MethodDecl),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodVisibility {
    Private,
    Protected,
    Public,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodDecl {
    pub name: String,
    pub params: Vec<TypedParam>,
    pub return_type: Option<String>,
    pub body: Vec<Stmt>,
    pub visibility: MethodVisibility,
    pub is_static: bool,
    pub is_intrinsic: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedParam {
    pub name: String,
    pub ty: Option<String>,
}
