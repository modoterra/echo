use crate::{CallArg, Expr, QualifiedName};
use echo_source::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Compile(CompileStmt),
    Echo(EchoStmt),
    FunctionCall(FunctionCallStmt),
    DynamicFunctionCall(DynamicFunctionCallStmt),
    FunctionDecl(FunctionDeclStmt),
    Assign(AssignStmt),
    CoalesceAssign(CoalesceAssignStmt),
    ListAssign(ListAssignStmt),
    Let(LetStmt),
    AssignRef(AssignRefStmt),
    Return(ReturnStmt),
    Throw(ThrowStmt),
    Yield(YieldStmt),
    Goto(GotoStmt),
    Label(LabelStmt),
    PhpDeclare(PhpDeclareStmt),
    Global(GlobalStmt),
    StaticVar(StaticVarStmt),
    Expr(ExprStmt),
    Namespace(NamespaceStmt),
    Use(UseStmt),
    Import(ImportStmt),
    UnnamedExport(UnnamedExportStmt),
    ClassDecl(ClassDeclStmt),
    InterfaceDecl(InterfaceDeclStmt),
    TraitDecl(TraitDeclStmt),
    EnumDecl(EnumDeclStmt),
    FacetDecl(FacetDeclStmt),
    TypeDecl(TypeDeclStmt),
    Loop(LoopStmt),
    While(WhileStmt),
    DoWhile(DoWhileStmt),
    For(ForStmt),
    Foreach(ForeachStmt),
    Switch(SwitchStmt),
    If(IfStmt),
    Try(TryStmt),
    Break(BreakStmt),
    Continue(ContinueStmt),
    Append(AppendStmt),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompileStmt {
    pub entries: Vec<CompileEntry>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileEntry {
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EchoStmt {
    pub exprs: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCallStmt {
    pub name: String,
    pub args: Vec<CallArg>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DynamicFunctionCallStmt {
    pub name: String,
    pub args: Vec<CallArg>,
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
pub struct CoalesceAssignStmt {
    pub name: String,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListAssignStmt {
    pub targets: Vec<String>,
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
    pub value: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThrowStmt {
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct YieldStmt {
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GotoStmt {
    pub label: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelStmt {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhpDeclareStmt {
    pub directives: Vec<PhpDeclareDirective>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhpDeclareDirective {
    pub name: String,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalStmt {
    pub names: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticVarStmt {
    pub vars: Vec<StaticVarDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticVarDecl {
    pub name: String,
    pub value: Option<Expr>,
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
    Echo,
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
pub struct UnnamedExportStmt {
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDeclStmt {
    pub name: String,
    pub modifiers: Vec<ClassModifier>,
    pub parent: Option<QualifiedName>,
    pub interfaces: Vec<QualifiedName>,
    pub members: Vec<ClassMember>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassModifier {
    Abstract,
    Final,
    Readonly,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceDeclStmt {
    pub name: String,
    pub parents: Vec<QualifiedName>,
    pub members: Vec<InterfaceMember>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitDeclStmt {
    pub name: String,
    pub members: Vec<ClassMember>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDeclStmt {
    pub name: String,
    pub backing_type: Option<String>,
    pub interfaces: Vec<QualifiedName>,
    pub members: Vec<EnumMember>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FacetDeclStmt {
    pub target: String,
    pub receiver: String,
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
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DoWhileStmt {
    pub body: Vec<Stmt>,
    pub condition: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForStmt {
    pub init: Vec<Expr>,
    pub conditions: Vec<Expr>,
    pub increments: Vec<Expr>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForeachStmt {
    pub iterable: Expr,
    pub key: Option<String>,
    pub value: String,
    pub value_by_ref: bool,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwitchStmt {
    pub expr: Expr,
    pub cases: Vec<SwitchCase>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwitchCase {
    pub condition: Option<Expr>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStmt {
    pub condition: Expr,
    pub body: Vec<Stmt>,
    pub elseif_clauses: Vec<ElseIfClause>,
    pub else_body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TryStmt {
    pub body: Vec<Stmt>,
    pub catches: Vec<CatchClause>,
    pub finally_body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatchClause {
    pub types: Vec<QualifiedName>,
    pub variable: Option<String>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ElseIfClause {
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
pub struct ContinueStmt {
    pub value: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AppendStmt {
    pub target: Expr,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClassMember {
    Method(MethodDecl),
    Property(PropertyDecl),
    Const(ClassConstDecl),
    TraitUse(QualifiedName),
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnumMember {
    Case(EnumCaseDecl),
    Method(MethodDecl),
    TraitUse(QualifiedName),
}

#[derive(Debug, Clone, PartialEq)]
pub enum InterfaceMember {
    Method(MethodDecl),
    Const(ClassConstDecl),
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumCaseDecl {
    pub name: String,
    pub value: Option<Expr>,
    pub span: Span,
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
    pub is_abstract: bool,
    pub is_final: bool,
    pub is_static: bool,
    pub is_intrinsic: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertyDecl {
    pub name: String,
    pub value: Option<Expr>,
    pub visibility: MethodVisibility,
    pub is_static: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassConstDecl {
    pub name: String,
    pub value: Expr,
    pub visibility: MethodVisibility,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypedParam {
    pub name: String,
    pub ty: Option<String>,
    pub default_value: Option<Expr>,
    pub promoted_visibility: Option<MethodVisibility>,
}
