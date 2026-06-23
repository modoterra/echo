use crate::Stmt;
use echo_source::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub open_tag: Option<Span>,
    pub statements: Vec<Stmt>,
    pub source_dir: Option<String>,
    pub span: Span,
}
