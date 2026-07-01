use crate::{ClassMember, ClassModifier, QualifiedName, Stmt};
use echo_source::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Null(NullLiteral),
    Bool(BoolLiteral),
    String(StringLiteral),
    Number(NumberLiteral),
    Variable(VariableExpr),
    Constant(ConstantExpr),
    ReceiverConst(ReceiverConstExpr),
    StaticPropertyFetch(Box<StaticPropertyFetchExpr>),
    StaticPropertyAssign(Box<StaticPropertyAssignExpr>),
    StaticPropertyCoalesceAssign(Box<StaticPropertyAssignExpr>),
    ClassConstantFetch(Box<ClassConstantFetchExpr>),
    FunctionCall(FunctionCallExpr),
    DynamicFunctionCall(DynamicFunctionCallExpr),
    DynamicCall(Box<DynamicCallExpr>),
    MethodCall(Box<MethodCallExpr>),
    StaticCall(StaticCallExpr),
    New(Box<NewExpr>),
    Closure(Box<ClosureExpr>),
    ArrowFunction(Box<ArrowFunctionExpr>),
    Assign(Box<AssignExpr>),
    MagicConstant(MagicConstantExpr),
    Include(Box<IncludeExpr>),
    Defer(DeferExpr),
    Run(RunExpr),
    Fork(ForkExpr),
    Spawn(SpawnExpr),
    Join(JoinExpr),
    Loop(LoopExpr),
    Unary(Box<UnaryExpr>),
    Cast(Box<CastExpr>),
    Binary(Box<BinaryExpr>),
    Ternary(Box<TernaryExpr>),
    Match(Box<MatchExpr>),
    TypeAscription(Box<TypeAscriptionExpr>),
    Field(Box<FieldExpr>),
    Index(Box<IndexExpr>),
    TargetAssign(Box<TargetAssignExpr>),
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
            Self::Constant(expr) => expr.span,
            Self::ReceiverConst(expr) => expr.span,
            Self::StaticPropertyFetch(expr) => expr.span,
            Self::StaticPropertyAssign(expr) => expr.span,
            Self::StaticPropertyCoalesceAssign(expr) => expr.span,
            Self::ClassConstantFetch(expr) => expr.span,
            Self::FunctionCall(expr) => expr.span,
            Self::DynamicFunctionCall(expr) => expr.span,
            Self::DynamicCall(expr) => expr.span,
            Self::MethodCall(expr) => expr.span,
            Self::StaticCall(expr) => expr.span,
            Self::New(expr) => expr.span,
            Self::Closure(expr) => expr.span,
            Self::ArrowFunction(expr) => expr.span,
            Self::Assign(expr) => expr.span,
            Self::MagicConstant(expr) => expr.span,
            Self::Include(expr) => expr.span,
            Self::Defer(expr) => expr.span,
            Self::Run(expr) => expr.span(),
            Self::Fork(expr) => expr.span(),
            Self::Spawn(expr) => expr.span,
            Self::Join(expr) => expr.span,
            Self::Loop(expr) => expr.span,
            Self::Unary(expr) => expr.span,
            Self::Cast(expr) => expr.span,
            Self::Binary(expr) => expr.span,
            Self::Ternary(expr) => expr.span,
            Self::Match(expr) => expr.span,
            Self::TypeAscription(expr) => expr.span,
            Self::Field(expr) => expr.span,
            Self::Index(expr) => expr.span,
            Self::TargetAssign(expr) => expr.span,
            Self::Object(expr) => expr.span,
            Self::List(expr) => expr.span,
            Self::Array(expr) => expr.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeAscriptionExpr {
    pub expr: Expr,
    pub ty: String,
    pub span: Span,
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
pub struct TargetAssignExpr {
    pub target: Expr,
    pub value: Expr,
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
pub struct ConstantExpr {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiverConst {
    This,
    SelfType,
    Parent,
    Static,
}

impl ReceiverConst {
    pub fn variable_name(self) -> &'static str {
        match self {
            Self::This => "this",
            Self::SelfType => "self",
            Self::Parent => "parent",
            Self::Static => "static",
        }
    }

    pub fn from_variable_name(name: &str) -> Option<Self> {
        match name {
            "this" => Some(Self::This),
            "self" => Some(Self::SelfType),
            "parent" => Some(Self::Parent),
            "static" => Some(Self::Static),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReceiverConstExpr {
    pub kind: ReceiverConst,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticPropertyFetchExpr {
    pub class_name: QualifiedName,
    pub property: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticPropertyAssignExpr {
    pub class_name: QualifiedName,
    pub property: String,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassConstantFetchExpr {
    pub class_name: QualifiedName,
    pub constant: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCallExpr {
    pub name: String,
    pub args: Vec<CallArg>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchExpr {
    pub subject: Expr,
    pub arms: Vec<MatchArm>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub conditions: Vec<Expr>,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DynamicFunctionCallExpr {
    pub name: String,
    pub args: Vec<CallArg>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DynamicCallExpr {
    pub callee: Expr,
    pub args: Vec<CallArg>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodCallExpr {
    pub object: Expr,
    pub method: String,
    pub method_span: Span,
    pub args: Vec<CallArg>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticCallExpr {
    pub class_name: QualifiedName,
    pub method: String,
    pub args: Vec<CallArg>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NewTarget {
    Class(QualifiedName),
    Expr(Box<Expr>),
    AnonymousClass(Box<AnonymousClassExpr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewExpr {
    pub target: NewTarget,
    pub args: Vec<CallArg>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnonymousClassExpr {
    pub modifiers: Vec<ClassModifier>,
    pub parent: Option<QualifiedName>,
    pub interfaces: Vec<QualifiedName>,
    pub members: Vec<ClassMember>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClosureExpr {
    pub params: Vec<crate::TypedParam>,
    pub captures: Vec<String>,
    pub return_type: Option<String>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrowFunctionExpr {
    pub params: Vec<crate::TypedParam>,
    pub return_type: Option<String>,
    pub body: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallArg {
    pub name: Option<String>,
    pub value: Expr,
    pub span: Span,
}

impl CallArg {
    pub fn positional(value: Expr) -> Self {
        Self {
            span: value.span(),
            value,
            name: None,
        }
    }
}

#[macro_export]
macro_rules! call_args {
    ($($value:expr),* $(,)?) => {
        vec![$($crate::CallArg::positional($value)),*]
    };
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
pub enum IncludeKind {
    Require,
    RequireOnce,
    Include,
    IncludeOnce,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IncludeExpr {
    pub kind: IncludeKind,
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
pub struct TernaryExpr {
    pub condition: Expr,
    pub if_true: Expr,
    pub if_false: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub expr: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CastExpr {
    pub ty: String,
    pub expr: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Plus,
    Minus,
    Not,
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
    LessThan,
    GreaterThanOrEqual,
    Identical,
    NotIdentical,
    Equal,
    NotEqual,
    InstanceOf,
    Coalesce,
    And,
    Or,
    Is,
    IsNot,
}
