use echo_index::{
    DefinitionLocation, DependencyFact, DependencyKind, EchoIndex, FileId, Symbol, SymbolKind,
    TextOffset,
};
use ropey::Rope;
use tower_lsp_server::ls_types::{Location, Uri};

use crate::position::range_to_lsp_range;

pub fn method_definition_at(
    index: &EchoIndex,
    text: &Rope,
    offset: TextOffset,
    symbols: &[&Symbol],
    dependencies: &[&DependencyFact],
) -> Option<DefinitionLocation> {
    let source = text.to_string();
    let method = method_call_at(&source, offset.0 as usize)?;
    let receiver_type = local_variable_type(symbols, &method.receiver)?;
    let class_name = resolve_imported_type(dependencies, &receiver_type).unwrap_or(receiver_type);
    index
        .method_definition(&class_name, &method.name)
        .map(DefinitionLocation::Symbol)
}

pub fn definition_location_to_lsp(
    index: &EchoIndex,
    text: &Rope,
    current_file_id: FileId,
    current_uri: &Uri,
    definition: DefinitionLocation,
) -> Option<Location> {
    let (file_id, range) = match definition {
        DefinitionLocation::Symbol(location) => (location.file_id, location.selection_range),
        DefinitionLocation::Dependency {
            file_id,
            selection_range,
            ..
        } => (file_id, selection_range),
    };

    if file_id == current_file_id {
        return Some(Location {
            uri: current_uri.clone(),
            range: range_to_lsp_range(text, range),
        });
    }

    let file = index.file(file_id)?;
    let uri = file.uri.parse::<Uri>().ok()?;
    let range = file
        .path
        .as_ref()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .map(|source| range_to_lsp_range(&Rope::from_str(&source), range))
        .unwrap_or_default();
    Some(Location { uri, range })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MethodCallAt {
    receiver: String,
    name: String,
}

fn method_call_at(source: &str, offset: usize) -> Option<MethodCallAt> {
    let offset = offset.min(source.len());
    let (name_start, name_end, name) = identifier_at(source, offset)?;
    if name_start < 2 || &source[name_start - 2..name_start] != "->" {
        return None;
    }
    let (receiver_start, _, receiver) = identifier_at(source, name_start.saturating_sub(3))?;
    if receiver_start == 0 || source.as_bytes().get(receiver_start - 1) != Some(&b'$') {
        return None;
    }
    if name_end < source.len()
        && source[name_end..]
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        return None;
    }
    Some(MethodCallAt { receiver, name })
}

fn identifier_at(source: &str, offset: usize) -> Option<(usize, usize, String)> {
    let bytes = source.as_bytes();
    if bytes.is_empty() || offset > bytes.len() {
        return None;
    }
    let mut start = offset.min(bytes.len().saturating_sub(1));
    while start > 0 && is_identifier_byte(bytes[start - 1]) {
        start -= 1;
    }
    let mut end = start;
    while end < bytes.len() && is_identifier_byte(bytes[end]) {
        end += 1;
    }
    if start == end || offset < start || offset > end {
        return None;
    }
    Some((start, end, source[start..end].to_string()))
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn local_variable_type(symbols: &[&Symbol], name: &str) -> Option<String> {
    symbols
        .iter()
        .find(|symbol| symbol.kind == SymbolKind::LocalVariable && symbol.name.text == name)
        .and_then(|symbol| {
            symbol
                .signature
                .as_ref()
                .map(|signature| signature.text.clone())
        })
}

fn resolve_imported_type(dependencies: &[&DependencyFact], ty: &str) -> Option<String> {
    dependencies
        .iter()
        .filter(|dependency| dependency.kind == DependencyKind::PhpUse)
        .find(|dependency| {
            dependency.alias.as_deref() == Some(ty)
                || dependency
                    .target
                    .rsplit('\\')
                    .next()
                    .is_some_and(|name| name == ty)
        })
        .map(|dependency| dependency.target.clone())
}

#[cfg(test)]
mod tests {
    use echo_index::{
        DefinitionLocation, EchoFileMode, EchoIndex, FileId, FqName, IndexFacts, IndexedFile,
        Signature, SymbolFact, SymbolKind, SymbolName, TextRange,
    };
    use tower_lsp_server::ls_types::{Position, Range};

    use super::*;

    #[test]
    fn converts_same_document_dependency_definition_location() {
        let mut index = EchoIndex::new();
        let file_id = FileId(1);
        let uri = "file:///project/public/index.php".parse::<Uri>().unwrap();
        index.insert_file(IndexedFile {
            file_id,
            uri: uri.to_string(),
            path: None,
            version: Some(1),
            mode: EchoFileMode::PhpCompat,
            content_hash: None,
        });

        let location = definition_location_to_lsp(
            &index,
            &Rope::from_str("<?php\nuse Illuminate\\Http\\Request;\n"),
            file_id,
            &uri,
            DefinitionLocation::Dependency {
                file_id,
                range: TextRange::new(6, 34),
                selection_range: TextRange::new(6, 34),
            },
        )
        .expect("location");

        assert_eq!(location.uri, uri);
        assert_eq!(
            location.range,
            Range {
                start: Position::new(1, 0),
                end: Position::new(1, 28),
            }
        );
    }

    #[test]
    fn resolves_phpdoc_receiver_method_to_indexed_class_method() {
        let mut index = EchoIndex::new();
        let source_file_id = FileId(1);
        let vendor_file_id = FileId(2);
        index.insert_file(IndexedFile {
            file_id: source_file_id,
            uri: "file:///project/public/index.php".to_string(),
            path: None,
            version: None,
            mode: EchoFileMode::PhpCompat,
            content_hash: None,
        });
        index.insert_file(IndexedFile {
            file_id: vendor_file_id,
            uri:
                "file:///project/vendor/laravel/framework/src/Illuminate/Foundation/Application.php"
                    .to_string(),
            path: None,
            version: None,
            mode: EchoFileMode::PhpCompat,
            content_hash: None,
        });
        index.update_file(
            vendor_file_id,
            IndexFacts::declarations(
                vendor_file_id,
                EchoFileMode::PhpCompat,
                vec![SymbolFact {
                    name: SymbolName::new("handleRequest"),
                    fq_name: Some(FqName::new(
                        vec!["Illuminate".into(), "Foundation".into()],
                        "Application::handleRequest",
                    )),
                    kind: SymbolKind::Method,
                    range: TextRange::new(60, 110),
                    selection_range: TextRange::new(76, 89),
                    visibility: None,
                    signature: None,
                }],
            ),
        );
        let app = echo_index::Symbol {
            id: echo_index::SymbolId(1),
            file_id: source_file_id,
            name: SymbolName::new("app"),
            fq_name: None,
            kind: SymbolKind::LocalVariable,
            range: TextRange::new(70, 95),
            selection_range: TextRange::new(91, 95),
            visibility: None,
            container: None,
            signature: Some(Signature {
                text: "Application".to_string(),
            }),
        };
        let dependency = DependencyFact {
            kind: DependencyKind::PhpUse,
            target: "Illuminate\\Foundation\\Application".to_string(),
            alias: None,
            range: TextRange::new(6, 43),
        };

        let definition = method_definition_at(
            &index,
            &Rope::from_str("<?php\n$app->handleRequest(Request::capture());\n"),
            TextOffset(13),
            &[&app],
            &[&dependency],
        )
        .expect("method definition");

        let DefinitionLocation::Symbol(location) = definition else {
            panic!("expected symbol definition");
        };
        assert_eq!(location.file_id, vendor_file_id);
        assert_eq!(location.selection_range, TextRange::new(76, 89));
    }
}
