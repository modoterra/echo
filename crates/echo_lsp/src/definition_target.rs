use std::path::Path;

use echo_index::{
    DependencyFact, DependencyKind, DependencyQuery, EchoFileMode, EchoIndex, FileId, IndexedFile,
    ReferenceKind, SymbolKind, TextRange,
};
use echo_source::SourceMode;
use ropey::Rope;
use tower_lsp_server::ls_types::{Location, Uri};

use crate::definition_composer::composer_class_file;
use crate::position::range_to_lsp_range;

pub(super) fn dependency_target_location(
    index: &mut EchoIndex,
    dependency: &DependencyFact,
) -> Option<Location> {
    match dependency.kind {
        DependencyKind::PhpUse => {
            let path = composer_class_file(index, &dependency.target)?;
            class_location(index, &path, &dependency.target)
        }
        DependencyKind::Require
        | DependencyKind::RequireOnce
        | DependencyKind::Include
        | DependencyKind::IncludeOnce
        | DependencyKind::ComposerAutoload => file_location(
            index,
            std::path::Path::new(&dependency.target),
            TextRange::new(0, 0),
        ),
        DependencyKind::EchoStdImport | DependencyKind::EchoFileImport => None,
    }
}

pub(super) fn reference_target_location(
    index: &mut EchoIndex,
    file_id: FileId,
    reference: echo_index::ReferenceFact,
) -> Option<Location> {
    match reference.kind {
        ReferenceKind::FilePath => file_location(
            index,
            std::path::Path::new(&reference.name),
            TextRange::new(0, 0),
        ),
        ReferenceKind::ClassLike => {
            let class_name = resolve_imported_class_name(index, file_id, &reference.name)
                .unwrap_or(reference.name);
            let path = composer_class_file(index, &class_name)?;
            class_location(index, &path, &class_name)
        }
        ReferenceKind::StaticMethod => {
            let class_name = reference.qualifier.as_deref()?;
            let class_name = resolve_imported_class_name(index, file_id, class_name)
                .unwrap_or_else(|| class_name.to_string());
            let path = composer_class_file(index, &class_name)?;
            index_php_file(index, &path);
            if let Some(location) = index.method_definition(&class_name, &reference.name) {
                return symbol_location(index, location.file_id, location.selection_range);
            }
            if let Some(range) = php_method_name_range(&path, &reference.name) {
                return file_location(index, &path, range);
            }
            file_location(index, &path, TextRange::new(0, 0))
        }
        ReferenceKind::Method => None,
    }
}

pub(super) fn class_location(
    index: &mut EchoIndex,
    path: &Path,
    class_name: &str,
) -> Option<Location> {
    let class_file_id = index_php_file(index, path)?;
    if let Some(location) = class_symbol_location(index, class_file_id, class_name) {
        return symbol_location(index, location.file_id, location.selection_range);
    }
    if let Some(range) = php_class_name_range(path, class_name) {
        return file_location(index, path, range);
    }
    file_location(index, path, TextRange::new(0, 0))
}

pub(super) fn file_location(
    index: &mut EchoIndex,
    path: &Path,
    range: TextRange,
) -> Option<Location> {
    let path = std::fs::canonicalize(path).ok()?;
    index_php_file(index, &path);
    let file_id = ensure_index_file(index, &path);
    symbol_location(index, file_id, range)
}

pub(super) fn symbol_location(
    index: &EchoIndex,
    file_id: FileId,
    range: TextRange,
) -> Option<Location> {
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

pub(super) fn index_php_file(index: &mut EchoIndex, path: &Path) -> Option<FileId> {
    let path = std::fs::canonicalize(path).ok()?;
    if path.extension().and_then(|ext| ext.to_str()) != Some("php") {
        return Some(ensure_index_file(index, &path));
    }
    let file_id = ensure_index_file(index, &path);
    if !index.document_symbols(file_id).is_empty() {
        return Some(file_id);
    }
    let source = std::fs::read_to_string(&path).ok()?;
    let Ok(mut program) = echo_parser::parse_with_mode(&source, SourceMode::Echo) else {
        return Some(file_id);
    };
    program.source_dir = path.parent().map(|path| path.to_string_lossy().to_string());
    let facts = echo_semantics::index_facts_from_source(
        &source,
        &program,
        file_id,
        EchoFileMode::PhpCompat,
    );
    index.update_file(file_id, facts);
    Some(file_id)
}

fn php_method_name_range(path: &Path, method_name: &str) -> Option<TextRange> {
    let source = std::fs::read_to_string(path).ok()?;
    let mut offset = 0;
    while let Some(relative) = source[offset..].find("function") {
        let function_start = offset + relative;
        if !is_php_word_boundary(&source, function_start, "function") {
            offset = function_start + "function".len();
            continue;
        }
        let mut name_start = function_start + "function".len();
        name_start += source[name_start..]
            .chars()
            .take_while(|ch| ch.is_whitespace())
            .map(char::len_utf8)
            .sum::<usize>();
        if source[name_start..].starts_with('&') {
            name_start += 1;
            name_start += source[name_start..]
                .chars()
                .take_while(|ch| ch.is_whitespace())
                .map(char::len_utf8)
                .sum::<usize>();
        }
        if source[name_start..].starts_with(method_name)
            && is_identifier_boundary(source.as_bytes().get(name_start + method_name.len()))
        {
            return Some(TextRange::new(
                name_start as u32,
                (name_start + method_name.len()) as u32,
            ));
        }
        offset = function_start + "function".len();
    }
    None
}

fn php_class_name_range(path: &Path, class_name: &str) -> Option<TextRange> {
    let short_name = class_name.rsplit('\\').next().unwrap_or(class_name);
    let source = std::fs::read_to_string(path).ok()?;
    let mut offset = 0;
    while let Some(relative) = source[offset..].find("class") {
        let class_start = offset + relative;
        if !is_php_word_boundary(&source, class_start, "class") {
            offset = class_start + "class".len();
            continue;
        }
        let mut name_start = class_start + "class".len();
        name_start += source[name_start..]
            .chars()
            .take_while(|ch| ch.is_whitespace())
            .map(char::len_utf8)
            .sum::<usize>();
        if source[name_start..].starts_with(short_name)
            && is_identifier_boundary(source.as_bytes().get(name_start + short_name.len()))
        {
            return Some(TextRange::new(
                name_start as u32,
                (name_start + short_name.len()) as u32,
            ));
        }
        offset = class_start + "class".len();
    }
    None
}

fn class_symbol_location(
    index: &EchoIndex,
    file_id: FileId,
    class_name: &str,
) -> Option<echo_index::SymbolLocation> {
    let short_name = class_name.rsplit('\\').next().unwrap_or(class_name);
    index
        .document_symbols(file_id)
        .into_iter()
        .find(|symbol| {
            symbol.kind == SymbolKind::Class
                && (symbol
                    .fq_name
                    .as_ref()
                    .is_some_and(|fq_name| fq_name.as_string() == class_name)
                    || symbol.name.text == short_name)
        })
        .map(|symbol| echo_index::SymbolLocation {
            file_id: symbol.file_id,
            symbol_id: symbol.id,
            range: symbol.range,
            selection_range: symbol.selection_range,
        })
}

fn is_php_word_boundary(source: &str, start: usize, word: &str) -> bool {
    let bytes = source.as_bytes();
    is_identifier_boundary(start.checked_sub(1).and_then(|index| bytes.get(index)))
        && is_identifier_boundary(bytes.get(start + word.len()))
}

fn is_identifier_boundary(byte: Option<&u8>) -> bool {
    byte.is_none_or(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_')
}

fn ensure_index_file(index: &mut EchoIndex, path: &Path) -> FileId {
    if let Some(file_id) = index.file_id_by_path(path) {
        return file_id;
    }

    let file_id = index.alloc_file_id();
    let uri = Uri::from_file_path(path)
        .map(|uri| uri.to_string())
        .unwrap_or_else(|| format!("file://{}", path.display()));
    index.insert_file(IndexedFile {
        file_id,
        uri,
        path: Some(path.to_path_buf()),
        version: None,
        mode: EchoFileMode::PhpCompat,
        content_hash: None,
    });
    file_id
}

pub(super) fn resolve_imported_class_name(
    index: &EchoIndex,
    file_id: FileId,
    name: &str,
) -> Option<String> {
    index
        .dependencies(DependencyQuery::in_file(file_id))
        .into_iter()
        .filter(|dependency| dependency.kind == DependencyKind::PhpUse)
        .find(|dependency| {
            dependency.alias.as_deref() == Some(name)
                || dependency
                    .target
                    .rsplit('\\')
                    .next()
                    .is_some_and(|target_name| target_name == name)
        })
        .map(|dependency| dependency.target.clone())
}
