use std::collections::HashSet;

use echo_index::{DependencyFact, DependencyKind, ReferenceFact, ReferenceKind, TextRange};
use ropey::Rope;
use tower_lsp_server::ls_types::{DocumentLink, Uri};

use crate::position::range_to_lsp_range;

pub(crate) fn document_links_for_paths(
    text: &Rope,
    dependencies: &[&DependencyFact],
    references: &[&ReferenceFact],
) -> Vec<DocumentLink> {
    let mut links = Vec::new();
    let mut ranges = HashSet::new();

    for dependency in dependencies {
        if !matches!(
            dependency.kind,
            DependencyKind::Require
                | DependencyKind::RequireOnce
                | DependencyKind::Include
                | DependencyKind::IncludeOnce
                | DependencyKind::Compile
                | DependencyKind::ComposerAutoload
        ) {
            continue;
        }
        push_document_link(
            text,
            &mut links,
            &mut ranges,
            dependency.target_range,
            &dependency.target,
        );
    }

    for reference in references {
        if reference.kind != ReferenceKind::FilePath {
            continue;
        }
        push_document_link(
            text,
            &mut links,
            &mut ranges,
            reference.range,
            &reference.name,
        );
    }

    links
}

fn push_document_link(
    text: &Rope,
    links: &mut Vec<DocumentLink>,
    ranges: &mut HashSet<TextRange>,
    range: TextRange,
    target: &str,
) {
    if !ranges.insert(range) {
        return;
    }
    let Ok(path) = std::fs::canonicalize(target) else {
        return;
    };
    let Some(uri) = Uri::from_file_path(path) else {
        return;
    };
    links.push(DocumentLink {
        range: range_to_lsp_range(text, range),
        target: Some(uri),
        tooltip: Some(target.to_string()),
        data: None,
    });
}
