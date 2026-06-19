use echo_index::{EchoIndex, Symbol, SymbolKind as EchoSymbolKind};
use ropey::Rope;
use tower_lsp_server::ls_types::{DocumentSymbol, Location, SymbolInformation, SymbolKind};
use tower_lsp_server::ls_types::{Range, Uri};

use crate::position::range_to_lsp_range;

#[allow(deprecated)]
pub fn document_symbols_to_lsp(text: &Rope, symbols: &[&Symbol]) -> Vec<DocumentSymbol> {
    symbols
        .iter()
        .map(|symbol| DocumentSymbol {
            name: symbol.name.text.to_string(),
            detail: symbol
                .signature
                .as_ref()
                .map(|signature| signature.text.clone()),
            kind: symbol_kind_to_lsp(symbol.kind),
            tags: None,
            deprecated: None,
            range: range_to_lsp_range(text, symbol.range),
            selection_range: range_to_lsp_range(text, symbol.selection_range),
            children: None,
        })
        .collect()
}

#[allow(deprecated)]
pub fn workspace_symbols_to_lsp(
    index: &EchoIndex,
    query: &str,
    limit: usize,
) -> Vec<SymbolInformation> {
    index
        .workspace_symbols(query, limit)
        .into_iter()
        .filter_map(|symbol| {
            let file = index.file(symbol.file_id)?;
            let uri = file.uri.parse::<Uri>().ok()?;
            Some(SymbolInformation {
                name: symbol.name.text.to_string(),
                kind: symbol_kind_to_lsp(symbol.kind),
                tags: None,
                deprecated: None,
                location: Location {
                    uri,
                    range: Range::default(),
                },
                container_name: symbol.fq_name.as_ref().and_then(|name| {
                    if name.namespace.is_empty() {
                        None
                    } else {
                        Some(name.namespace.join("\\"))
                    }
                }),
            })
        })
        .collect()
}

pub(crate) fn symbol_kind_to_lsp(kind: EchoSymbolKind) -> SymbolKind {
    match kind {
        EchoSymbolKind::Function => SymbolKind::FUNCTION,
        EchoSymbolKind::Method => SymbolKind::METHOD,
        EchoSymbolKind::Class => SymbolKind::CLASS,
        EchoSymbolKind::Interface => SymbolKind::INTERFACE,
        EchoSymbolKind::Trait => SymbolKind::STRUCT,
        EchoSymbolKind::Enum => SymbolKind::ENUM,
        EchoSymbolKind::Constant => SymbolKind::CONSTANT,
        EchoSymbolKind::Property => SymbolKind::PROPERTY,
        EchoSymbolKind::Parameter | EchoSymbolKind::LocalVariable => SymbolKind::VARIABLE,
        EchoSymbolKind::Namespace => SymbolKind::NAMESPACE,
        EchoSymbolKind::TypeAlias => SymbolKind::TYPE_PARAMETER,
        EchoSymbolKind::ErrorType => SymbolKind::OBJECT,
        EchoSymbolKind::Extension => SymbolKind::MODULE,
    }
}

#[cfg(test)]
mod tests {
    use echo_index::{
        EchoFileMode, EchoIndex, FileId, IndexFacts, IndexedFile, Signature, Symbol, SymbolFact,
        SymbolId, SymbolKind as EchoSymbolKind, SymbolName, TextRange,
    };

    use super::*;

    #[test]
    fn converts_index_symbols_to_document_symbols() {
        let text = Rope::from_str("<?php\nfn handler(): string;\n");
        let symbol = Symbol {
            id: SymbolId(1),
            file_id: FileId(1),
            name: SymbolName::new("handler"),
            fq_name: None,
            kind: EchoSymbolKind::Function,
            range: TextRange::new(6, 27),
            selection_range: TextRange::new(9, 16),
            visibility: None,
            container: None,
            signature: Some(Signature {
                text: "(): string".to_string(),
            }),
        };

        let symbols = document_symbols_to_lsp(&text, &[&symbol]);

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "handler");
        assert_eq!(symbols[0].kind, SymbolKind::FUNCTION);
        assert_eq!(symbols[0].detail.as_deref(), Some("(): string"));
    }

    #[test]
    fn converts_index_symbols_to_workspace_symbols() {
        let mut index = EchoIndex::new();
        let file_id = index.alloc_file_id();
        index.insert_file(IndexedFile {
            file_id,
            uri: "file:///project/public/index.php".to_string(),
            path: None,
            version: Some(1),
            mode: EchoFileMode::PhpCompat,
            content_hash: None,
        });
        index.update_file(
            file_id,
            IndexFacts::declarations(
                file_id,
                EchoFileMode::PhpCompat,
                vec![SymbolFact {
                    name: SymbolName::new("Application"),
                    fq_name: None,
                    kind: EchoSymbolKind::Class,
                    range: TextRange::new(0, 20),
                    selection_range: TextRange::new(0, 11),
                    visibility: None,
                    signature: None,
                }],
            ),
        );

        let symbols = workspace_symbols_to_lsp(&index, "App", 10);

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "Application");
        assert_eq!(symbols[0].kind, SymbolKind::CLASS);
        assert_eq!(
            symbols[0].location.uri.to_string(),
            "file:///project/public/index.php"
        );
    }
}
