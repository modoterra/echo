use crate::Stmt;
use echo_source::{SourceId, SourceSpan, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub open_tag: Option<Span>,
    pub statements: Vec<Stmt>,
    pub source_id: Option<SourceId>,
    pub source_dir: Option<String>,
    pub span: Span,
}

impl Program {
    pub fn source_span(&self, span: Span) -> Option<SourceSpan> {
        self.source_id
            .map(|source_id| SourceSpan::new(source_id, span))
    }
}
