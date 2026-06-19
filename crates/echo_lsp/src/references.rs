use echo_index::{FileId, ReferenceLocation};
use ropey::Rope;
use tower_lsp_server::ls_types::{Location, Uri};

use crate::position::range_to_lsp_range;

pub fn reference_locations_to_lsp(
    text: &Rope,
    current_file_id: FileId,
    current_uri: &Uri,
    references: &[ReferenceLocation],
) -> Vec<Location> {
    references
        .iter()
        .filter(|reference| reference.file_id == current_file_id)
        .map(|reference| Location {
            uri: current_uri.clone(),
            range: range_to_lsp_range(text, reference.range),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use echo_index::{FileId, ReferenceLocation, TextRange};
    use tower_lsp_server::ls_types::{Position, Range};

    use super::*;

    #[test]
    fn converts_same_document_reference_locations() {
        let uri = "file:///project/public/index.php".parse::<Uri>().unwrap();
        let locations = reference_locations_to_lsp(
            &Rope::from_str("<?php\nRequest::capture();\n"),
            FileId(1),
            &uri,
            &[ReferenceLocation {
                file_id: FileId(1),
                range: TextRange::new(6, 13),
            }],
        );

        assert_eq!(locations.len(), 1);
        assert_eq!(
            locations[0].range,
            Range {
                start: Position::new(1, 0),
                end: Position::new(1, 7),
            }
        );
    }
}
