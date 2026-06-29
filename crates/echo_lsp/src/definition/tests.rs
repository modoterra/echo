use echo_index::{
    DefinitionLocation, DependencyFact, DependencyKind, EchoIndex, FileId, FqName, IndexFacts,
    IndexedFile, ReferenceFact, ReferenceKind, Signature, SymbolFact, SymbolKind, SymbolName,
    TextRange,
};
use tower_lsp_server::ls_types::{Position, Range};

use super::*;

#[path = "tests/file_paths.rs"]
mod file_paths;
#[path = "tests/methods.rs"]
mod methods;
#[path = "tests/targets.rs"]
mod targets;

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
