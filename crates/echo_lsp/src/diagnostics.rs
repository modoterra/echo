use echo_diagnostics::Diagnostic as EchoDiagnostic;
use echo_index::TextRange;
use ropey::Rope;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity};

use crate::position::range_to_lsp_range;

pub fn diagnostics_to_lsp(text: &Rope, diagnostics: &[EchoDiagnostic]) -> Vec<Diagnostic> {
    diagnostics
        .iter()
        .map(|diagnostic| Diagnostic {
            range: range_to_lsp_range(
                text,
                TextRange {
                    start: diagnostic.span.start as u32,
                    end: diagnostic.span.end as u32,
                },
            ),
            severity: Some(DiagnosticSeverity::ERROR),
            code: None,
            code_description: None,
            source: Some("echo".to_owned()),
            message: diagnostic.message.clone(),
            related_information: None,
            tags: None,
            data: None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use echo_source::Span;
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
}
