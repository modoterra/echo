use echo_source::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub open_tag: Option<Span>,
    pub statements: Vec<Stmt>,
    pub source_dir: Option<String>,
    pub span: Span,
}

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

#[derive(Debug, Clone, PartialEq)]
pub struct MethodDecl {
    pub name: String,
    pub params: Vec<TypedParam>,
    pub return_type: Option<String>,
    pub is_static: bool,
    pub is_intrinsic: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedParam {
    pub name: String,
    pub ty: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualifiedName {
    pub parts: Vec<String>,
}

impl QualifiedName {
    pub fn new(parts: Vec<String>) -> Self {
        Self { parts }
    }

    pub fn as_string(&self) -> String {
        self.parts.join("\\")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Null(NullLiteral),
    Bool(BoolLiteral),
    String(StringLiteral),
    Number(NumberLiteral),
    Variable(VariableExpr),
    FunctionCall(FunctionCallExpr),
    MethodCall(Box<MethodCallExpr>),
    StaticCall(StaticCallExpr),
    Assign(Box<AssignExpr>),
    MagicConstant(MagicConstantExpr),
    Require(Box<RequireExpr>),
    Defer(DeferExpr),
    Run(RunExpr),
    Fork(ForkExpr),
    Spawn(SpawnExpr),
    Join(JoinExpr),
    Loop(LoopExpr),
    Unary(Box<UnaryExpr>),
    Binary(Box<BinaryExpr>),
    Field(Box<FieldExpr>),
    Index(Box<IndexExpr>),
    Object(ObjectExpr),
    List(ListExpr),
    Array(ArrayExpr),
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Self::Null(expr) => expr.span,
            Self::Bool(expr) => expr.span,
            Self::String(expr) => expr.span,
            Self::Number(expr) => expr.span,
            Self::Variable(expr) => expr.span,
            Self::FunctionCall(expr) => expr.span,
            Self::MethodCall(expr) => expr.span,
            Self::StaticCall(expr) => expr.span,
            Self::Assign(expr) => expr.span,
            Self::MagicConstant(expr) => expr.span,
            Self::Require(expr) => expr.span,
            Self::Defer(expr) => expr.span,
            Self::Run(expr) => expr.span(),
            Self::Fork(expr) => expr.span(),
            Self::Spawn(expr) => expr.span,
            Self::Join(expr) => expr.span,
            Self::Loop(expr) => expr.span,
            Self::Unary(expr) => expr.span,
            Self::Binary(expr) => expr.span,
            Self::Field(expr) => expr.span,
            Self::Index(expr) => expr.span,
            Self::Object(expr) => expr.span,
            Self::List(expr) => expr.span,
            Self::Array(expr) => expr.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldExpr {
    pub object: Expr,
    pub field: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndexExpr {
    pub collection: Expr,
    pub index: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectExpr {
    pub name: String,
    pub fields: Vec<ObjectField>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectField {
    pub name: String,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListExpr {
    pub values: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayExpr {
    pub elements: Vec<ArrayElement>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayElement {
    pub key: Option<Expr>,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NullLiteral {
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoolLiteral {
    pub value: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringLiteral {
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberLiteral {
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableExpr {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCallExpr {
    pub name: String,
    pub args: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodCallExpr {
    pub object: Expr,
    pub method: String,
    pub args: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticCallExpr {
    pub class_name: QualifiedName,
    pub method: String,
    pub args: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssignExpr {
    pub name: String,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MagicConstantKind {
    Dir,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MagicConstantExpr {
    pub kind: MagicConstantKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequireKind {
    Require,
    RequireOnce,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RequireExpr {
    pub kind: RequireKind,
    pub path: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeferExpr {
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RunExpr {
    Block { body: Vec<Stmt>, span: Span },
    Task { expr: Box<Expr>, span: Span },
    Group { entries: Vec<Vec<Stmt>>, span: Span },
}

impl RunExpr {
    pub fn span(&self) -> Span {
        match self {
            Self::Block { span, .. } | Self::Task { span, .. } | Self::Group { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForkExpr {
    Block { body: Vec<Stmt>, span: Span },
    Task { expr: Box<Expr>, span: Span },
}

impl ForkExpr {
    pub fn span(&self) -> Span {
        match self {
            Self::Block { span, .. } | Self::Task { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpawnExpr {
    pub command: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JoinExpr {
    pub handle: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoopExpr {
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
    pub left: Expr,
    pub op: BinaryOp,
    pub right: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub expr: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Plus,
    Minus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Concat,
    Is,
    IsNot,
}
