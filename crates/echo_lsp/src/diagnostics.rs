use echo_diagnostics::Diagnostic as EchoDiagnostic;
use echo_index::TextRange;
use ropey::Rope;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity};

use crate::position::range_to_lsp_range;
use echo_source::SourceId;

pub fn diagnostics_to_lsp(text: &Rope, diagnostics: &[EchoDiagnostic]) -> Vec<Diagnostic> {
    diagnostics_to_lsp_for_source(text, None, diagnostics)
}

pub fn diagnostics_to_lsp_for_source(
    text: &Rope,
    source_id: Option<SourceId>,
    diagnostics: &[EchoDiagnostic],
) -> Vec<Diagnostic> {
    diagnostics
        .iter()
        .filter_map(|diagnostic| {
            diagnostic_span_for_source(diagnostic, source_id).map(|span| (diagnostic, span))
        })
        .map(|diagnostic| Diagnostic {
            range: range_to_lsp_range(
                text,
                TextRange {
                    start: diagnostic.1.start as u32,
                    end: diagnostic.1.end as u32,
                },
            ),
            severity: Some(DiagnosticSeverity::ERROR),
            code: None,
            code_description: None,
            source: Some("echo".to_owned()),
            message: diagnostic.0.message.clone(),
            related_information: None,
            tags: None,
            data: None,
        })
        .collect()
}

fn diagnostic_span_for_source(
    diagnostic: &EchoDiagnostic,
    source_id: Option<SourceId>,
) -> Option<echo_source::Span> {
    match (source_id, diagnostic.source_span) {
        (Some(source_id), Some(source_span)) if source_span.source_id == source_id => {
            Some(source_span.span)
        }
        (Some(_), Some(_)) => None,
        _ => Some(diagnostic.span),
    }
}

#[cfg(test)]
mod tests {
    use echo_source::{SourceId, SourceSpan, Span};
    use tower_lsp_server::ls_types::{Position, Range};

    use super::*;

    #[test]
    fn converts_echo_diagnostics_to_lsp_diagnostics() {
        let text = Rope::from_str("echo \"ok\"\nerror\n");
        let diagnostics = diagnostics_to_lsp(
            &text,
            &[EchoDiagnostic::new("expected statement", Span::new(10, 15))],
        );

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "expected statement");
        assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(
            diagnostics[0].range,
            Range {
                start: Position::new(1, 0),
                end: Position::new(1, 5),
            }
        );
    }

    #[test]
    fn converts_only_matching_source_diagnostics_when_source_id_is_known() {
        let text = Rope::from_str("echo \"ok\"\nerror\n");
        let source = SourceId::new(1);
        let other_source = SourceId::new(2);
        let diagnostics = diagnostics_to_lsp_for_source(
            &text,
            Some(source),
            &[
                EchoDiagnostic::new_at_source(
                    "expected statement",
                    SourceSpan::new(source, Span::new(10, 15)),
                ),
                EchoDiagnostic::new_at_source(
                    "other file",
                    SourceSpan::new(other_source, Span::new(0, 4)),
                ),
            ],
        );

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "expected statement");
    }
}
