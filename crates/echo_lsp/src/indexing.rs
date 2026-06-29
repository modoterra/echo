use std::collections::HashSet;
use std::path::{Path, PathBuf};

use echo_diagnostics::Diagnostic as EchoDiagnostic;
use echo_index::{DependencyKind, DependencyQuery, EchoIndex, FileId, IndexFacts, IndexedFile};
use tower_lsp_server::ls_types::Uri;

pub(crate) fn parse_index_facts(
    source: &str,
    file_id: FileId,
    source_dir: Option<&Path>,
) -> std::result::Result<IndexFacts, Vec<EchoDiagnostic>> {
    let mut program = echo_parser::parse(source)?;
    program.source_dir = source_dir.map(|path| path.to_string_lossy().to_string());
    Ok(echo_semantics::index_facts_from_source(
        source, &program, file_id,
    ))
}

pub(crate) fn index_required_files(index: &mut EchoIndex, root_file_id: FileId) {
    let mut visited = HashSet::new();
    index_required_files_inner(index, root_file_id, &mut visited);
}

fn index_required_files_inner(
    index: &mut EchoIndex,
    file_id: FileId,
    visited: &mut HashSet<PathBuf>,
) {
    let dependencies = index
        .dependencies(DependencyQuery::in_file(file_id))
        .into_iter()
        .filter(|dependency| {
            matches!(
                dependency.kind,
                DependencyKind::Require
                    | DependencyKind::RequireOnce
                    | DependencyKind::Include
                    | DependencyKind::IncludeOnce
                    | DependencyKind::ComposerAutoload
            )
        })
        .map(|dependency| dependency.target.clone())
        .collect::<Vec<_>>();

    for target in dependencies {
        let path = PathBuf::from(target);
        let Ok(path) = std::fs::canonicalize(path) else {
            continue;
        };
        if !visited.insert(path.clone())
            || path.extension().and_then(|ext| ext.to_str()) != Some("php")
        {
            continue;
        }

        let included_file_id = match index.file_by_path(&path).map(|file| file.file_id) {
            Some(file_id) => file_id,
            None => {
                let file_id = index.alloc_file_id();
                let uri = Uri::from_file_path(&path)
                    .map(|uri| uri.to_string())
                    .unwrap_or_else(|| format!("file://{}", path.display()));
                index.insert_file(IndexedFile {
                    file_id,
                    uri,
                    path: Some(path.clone()),
                    version: None,
                    content_hash: None,
                });
                file_id
            }
        };

        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        let source_dir = path.parent();
        if let Ok(facts) = parse_index_facts(&source, included_file_id, source_dir) {
            index.update_file(included_file_id, facts);
            index_required_files_inner(index, included_file_id, visited);
        }
    }
}
