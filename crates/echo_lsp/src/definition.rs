use echo_index::{DefinitionLocation, EchoIndex, FileId};
use ropey::Rope;
use tower_lsp_server::ls_types::{Location, Uri};

use crate::position::range_to_lsp_range;

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
    Some(Location {
        uri,
        range: Default::default(),
    })
}

#[cfg(test)]
mod tests {
    use echo_index::{DefinitionLocation, EchoFileMode, EchoIndex, FileId, IndexedFile, TextRange};
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
}
