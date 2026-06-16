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
    FunctionDecl(FunctionDeclStmt),
    Assign(AssignStmt),
    AssignRef(AssignRefStmt),
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
pub enum Expr {
    Null(NullLiteral),
    String(StringLiteral),
    Number(NumberLiteral),
    Variable(VariableExpr),
    FunctionCall(FunctionCallExpr),
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
