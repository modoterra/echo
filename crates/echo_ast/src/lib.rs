use echo_source::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub open_tag: Option<Span>,
    pub statements: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Echo(EchoStmt),
    FunctionCall(FunctionCallStmt),
    DynamicFunctionCall(DynamicFunctionCallStmt),
    FunctionDecl(FunctionDeclStmt),
    Assign(AssignStmt),
    AssignRef(AssignRefStmt),
    Return(ReturnStmt),
    Namespace(NamespaceStmt),
    Use(UseStmt),
    Import(ImportStmt),
    ClassDecl(ClassDeclStmt),
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
    pub params: Vec<String>,
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
    String(StringLiteral),
    Number(NumberLiteral),
    Variable(VariableExpr),
    FunctionCall(FunctionCallExpr),
    Defer(DeferExpr),
    Run(RunExpr),
    Fork(ForkExpr),
    Spawn(SpawnExpr),
    Join(JoinExpr),
    Binary(Box<BinaryExpr>),
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Self::Null(expr) => expr.span,
            Self::String(expr) => expr.span,
            Self::Number(expr) => expr.span,
            Self::Variable(expr) => expr.span,
            Self::FunctionCall(expr) => expr.span,
            Self::Defer(expr) => expr.span,
            Self::Run(expr) => expr.span(),
            Self::Fork(expr) => expr.span(),
            Self::Spawn(expr) => expr.span,
            Self::Join(expr) => expr.span,
            Self::Binary(expr) => expr.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NullLiteral {
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
pub struct DeferExpr {
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RunExpr {
    Block { body: Vec<Stmt>, span: Span },
    Task { expr: Box<Expr>, span: Span },
}

impl RunExpr {
    pub fn span(&self) -> Span {
        match self {
            Self::Block { span, .. } | Self::Task { span, .. } => *span,
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
pub struct BinaryExpr {
    pub left: Expr,
    pub op: BinaryOp,
    pub right: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Concat,
}
