use echo_index::{
    DependencyFact, DependencyKind, ReferenceFact, ReferenceKind, Symbol,
    SymbolKind as EchoSymbolKind, TextOffset,
};
use ropey::Rope;
use tower_lsp_server::ls_types::{Hover, HoverContents, MarkupContent, MarkupKind};

use crate::position::range_to_lsp_range;

pub fn hover_at(
    text: &Rope,
    offset: TextOffset,
    symbols: &[&Symbol],
    dependencies: &[&DependencyFact],
    references: &[&ReferenceFact],
) -> Option<Hover> {
    if let Some(symbol) = symbols
        .iter()
        .copied()
        .find(|symbol| symbol.selection_range.contains(offset))
        .or_else(|| {
            symbols
                .iter()
                .copied()
                .find(|symbol| symbol.range.contains(offset))
        })
    {
        return Some(markup_hover(
            symbol_hover_text(symbol),
            Some(range_to_lsp_range(text, symbol.selection_range)),
        ));
    }

    if let Some(variable_name) = variable_name_at(text, offset) {
        if let Some(symbol) = symbols.iter().copied().find(|symbol| {
            symbol.kind == EchoSymbolKind::LocalVariable && symbol.name.text == variable_name
        }) {
            return Some(markup_hover(
                symbol_hover_text(symbol),
                Some(range_to_lsp_range(text, symbol.selection_range)),
            ));
        }
    }

    dependencies
        .iter()
        .copied()
        .find(|dependency| dependency.target_range.contains(offset))
        .map(|dependency| {
            markup_hover(
                dependency_hover_text(dependency),
                Some(range_to_lsp_range(text, dependency.target_range)),
            )
        })
        .or_else(|| {
            references
                .iter()
                .find(|reference| reference.range.contains(offset))
                .map(|reference| {
                    markup_hover(
                        reference_hover_text(reference),
                        Some(range_to_lsp_range(text, reference.range)),
                    )
                })
        })
}

fn variable_name_at(text: &Rope, offset: TextOffset) -> Option<String> {
    let source = text.to_string();
    let offset = offset.0 as usize;
    if offset > source.len() {
        return None;
    }

    let bytes = source.as_bytes();
    let mut start = offset.min(bytes.len());
    while start > 0 && is_variable_name_byte(bytes[start - 1]) {
        start -= 1;
    }
    if start > 0 && bytes[start - 1] == b'$' {
        start -= 1;
    }
    if bytes.get(start) != Some(&b'$') {
        return None;
    }

    let mut end = start + 1;
    while end < bytes.len() && is_variable_name_byte(bytes[end]) {
        end += 1;
    }
    if end == start + 1 || offset < start || offset > end {
        return None;
    }

    Some(source[start + 1..end].to_string())
}

fn is_variable_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn markup_hover(value: String, range: Option<tower_lsp_server::ls_types::Range>) -> Hover {
    Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value,
        }),
        range,
    }
}

fn symbol_hover_text(symbol: &Symbol) -> String {
    let mut lines = vec![format!(
        "{} `{}`",
        symbol_kind_label(symbol.kind),
        symbol.name.text
    )];

    if let Some(signature) = &symbol.signature {
        lines.push(format!("signature `{}`", signature.text));
    }

    if let Some(fq_name) = &symbol.fq_name {
        lines.push(format!("fully qualified `{}`", fq_name.as_string()));
    }

    lines.join("\n\n")
}

fn dependency_hover_text(dependency: &DependencyFact) -> String {
    let label = match dependency.kind {
        DependencyKind::PhpUse => "PHP use",
        DependencyKind::EchoStdImport => "Echo std import",
        DependencyKind::EchoFileImport => "Echo file import",
        DependencyKind::Require => "PHP require",
        DependencyKind::RequireOnce => "PHP require_once",
        DependencyKind::Include => "PHP include",
        DependencyKind::IncludeOnce => "PHP include_once",
        DependencyKind::Compile => "Echo compile entry",
        DependencyKind::ComposerAutoload => "Composer autoload",
    };

    let mut lines = vec![format!("{label} `{}`", dependency.target)];
    if let Some(alias) = &dependency.alias {
        lines.push(format!("alias `{alias}`"));
    }
    lines.join("\n\n")
}

fn reference_hover_text(reference: &ReferenceFact) -> String {
    match reference.kind {
        ReferenceKind::ClassLike => format!("class reference `{}`", reference.name),
        ReferenceKind::Method => {
            if let Some(receiver) = &reference.qualifier {
                format!("method reference `${receiver}->{}`", reference.name)
            } else {
                format!("method reference `{}`", reference.name)
            }
        }
        ReferenceKind::StaticMethod => {
            if let Some(class_name) = &reference.qualifier {
                format!("static method reference `{class_name}::{}`", reference.name)
            } else {
                format!("static method reference `{}`", reference.name)
            }
        }
        ReferenceKind::FilePath => format!("file path `{}`", reference.name),
    }
}

fn symbol_kind_label(kind: EchoSymbolKind) -> &'static str {
    match kind {
        EchoSymbolKind::Function => "function",
        EchoSymbolKind::Method => "method",
        EchoSymbolKind::Class => "class",
        EchoSymbolKind::Interface => "interface",
        EchoSymbolKind::Trait => "trait",
        EchoSymbolKind::Enum => "enum",
        EchoSymbolKind::Constant => "constant",
        EchoSymbolKind::Property => "property",
        EchoSymbolKind::Parameter => "parameter",
        EchoSymbolKind::LocalVariable => "variable",
        EchoSymbolKind::Namespace => "namespace",
        EchoSymbolKind::TypeAlias => "type",
        EchoSymbolKind::ErrorType => "error type",
        EchoSymbolKind::Facet => "facet",
    }
}

#[cfg(test)]
mod tests {
    use echo_index::{
        DependencyFact, FileId, Signature, Symbol, SymbolId, SymbolKind, SymbolName, TextRange,
    };

    use super::*;

    #[test]
    fn returns_symbol_hover() {
        let text = Rope::from_str("<?php\nfn handler(): string;\n");
        let symbol = Symbol {
            id: SymbolId(1),
            file_id: FileId(1),
            name: SymbolName::new("handler"),
            fq_name: None,
            kind: SymbolKind::Function,
            range: TextRange::new(6, 27),
            selection_range: TextRange::new(9, 16),
            visibility: None,
            container: None,
            signature: Some(Signature {
                text: "(): string".to_string(),
            }),
        };

        let hover = hover_at(&text, TextOffset(10), &[&symbol], &[], &[]).expect("hover");

        let HoverContents::Markup(markup) = hover.contents else {
            panic!("expected markup hover");
        };
        assert!(markup.value.contains("function `handler`"));
        assert!(markup.value.contains("signature `(): string`"));
    }

    #[test]
    fn returns_dependency_hover() {
        let text = Rope::from_str("<?php\nuse Acme\\Http\\Request;\n");
        let dependency = DependencyFact {
            kind: DependencyKind::PhpUse,
            target: "Acme\\Http\\Request".to_string(),
            alias: None,
            range: TextRange::new(6, 28),
            target_range: TextRange::new(10, 27),
        };

        let hover = hover_at(&text, TextOffset(10), &[], &[&dependency], &[]).expect("hover");

        let HoverContents::Markup(markup) = hover.contents else {
            panic!("expected markup hover");
        };
        assert_eq!(markup.value, "PHP use `Acme\\Http\\Request`");
        assert_eq!(
            hover.range.expect("hover range"),
            tower_lsp_server::ls_types::Range {
                start: tower_lsp_server::ls_types::Position::new(1, 4),
                end: tower_lsp_server::ls_types::Position::new(1, 21),
            }
        );
    }

    #[test]
    fn returns_phpdoc_local_variable_hover_for_variable_usage() {
        let text = Rope::from_str("/** @var Kernel $app */\n$app->dispatch();\n");
        let symbol = Symbol {
            id: SymbolId(2),
            file_id: FileId(1),
            name: SymbolName::new("app"),
            fq_name: None,
            kind: SymbolKind::LocalVariable,
            range: TextRange::new(8, 20),
            selection_range: TextRange::new(16, 20),
            visibility: None,
            container: None,
            signature: Some(Signature {
                text: "Kernel".to_string(),
            }),
        };

        let hover = hover_at(&text, TextOffset(25), &[&symbol], &[], &[]).expect("hover");

        let HoverContents::Markup(markup) = hover.contents else {
            panic!("expected markup hover");
        };
        assert!(markup.value.contains("variable `app`"));
        assert!(markup.value.contains("signature `Kernel`"));
    }
}
