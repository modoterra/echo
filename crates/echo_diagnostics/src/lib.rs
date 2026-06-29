use echo_source::{SourceId, SourceSpan, Span};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: String,
    pub span: Span,
    pub source_span: Option<SourceSpan>,
}

impl Diagnostic {
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
            source_span: None,
        }
    }

    pub fn new_at_source(message: impl Into<String>, source_span: SourceSpan) -> Self {
        Self {
            message: message.into(),
            span: source_span.span,
            source_span: Some(source_span),
        }
    }

    pub fn with_source_id(self, source_id: SourceId) -> Self {
        Self::new_at_source(self.message, SourceSpan::new(source_id, self.span))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_can_carry_source_span_without_losing_local_span() {
        let source_span = SourceSpan::new(SourceId::new(7), Span::new(3, 8));
        let diagnostic = Diagnostic::new_at_source("expected expression", source_span);

        assert_eq!(diagnostic.span, Span::new(3, 8));
        assert_eq!(diagnostic.source_span, Some(source_span));
    }

    #[test]
    fn bare_diagnostic_can_be_attached_to_source_later() {
        let diagnostic = Diagnostic::new("expected statement", Span::new(10, 15))
            .with_source_id(SourceId::new(2));

        assert_eq!(
            diagnostic.source_span,
            Some(SourceSpan::new(SourceId::new(2), Span::new(10, 15)))
        );
    }
}
