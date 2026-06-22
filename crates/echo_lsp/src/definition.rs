use std::path::Path;

use echo_index::{
    DefinitionLocation, DependencyFact, DependencyKind, DependencyQuery, EchoFileMode, EchoIndex,
    FileId, IndexedFile, ReferenceKind, ReferenceQuery, Symbol, SymbolKind, TextOffset, TextRange,
};
use echo_source::SourceMode;
use ropey::Rope;
use tower_lsp_server::ls_types::{Location, LocationLink, Uri};

use crate::definition_composer::composer_class_file;
use crate::position::range_to_lsp_range;

pub fn method_definition_at(
    index: &EchoIndex,
    text: &Rope,
    offset: TextOffset,
    symbols: &[Symbol],
    dependencies: &[DependencyFact],
) -> Option<DefinitionLocation> {
    let source = text.to_string();
    let method = method_call_at(&source, offset.0 as usize)?;
    let receiver_type = local_variable_type(symbols, &method.receiver)?;
    let class_name = resolve_imported_type(dependencies, &receiver_type).unwrap_or(receiver_type);
    index
        .method_definition(&class_name, &method.name)
        .map(DefinitionLocation::Symbol)
}

#[cfg(test)]
pub fn dependency_target_location_at(
    index: &mut EchoIndex,
    file_id: FileId,
    offset: TextOffset,
) -> Option<Location> {
    let dependency = index
        .dependencies(DependencyQuery::in_file(file_id))
        .into_iter()
        .find(|dependency| dependency.target_range.contains(offset))?
        .clone();

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

pub fn dependency_target_link_at(
    index: &mut EchoIndex,
    text: &Rope,
    file_id: FileId,
    offset: TextOffset,
) -> Option<LocationLink> {
    let dependency = index
        .dependencies(DependencyQuery::in_file(file_id))
        .into_iter()
        .find(|dependency| dependency.target_range.contains(offset))?
        .clone();
    let location = dependency_target_location(index, &dependency)?;
    Some(location_to_link(text, dependency.target_range, location))
}

#[cfg(test)]
pub fn reference_target_location_at(
    index: &mut EchoIndex,
    file_id: FileId,
    offset: TextOffset,
) -> Option<Location> {
    let reference = index
        .references(ReferenceQuery::at(file_id, offset))
        .into_iter()
        .next()?
        .clone();

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

pub fn reference_target_link_at(
    index: &mut EchoIndex,
    text: &Rope,
    file_id: FileId,
    offset: TextOffset,
) -> Option<LocationLink> {
    let reference = index
        .references(ReferenceQuery::at(file_id, offset))
        .into_iter()
        .next()?
        .clone();
    let location = reference_target_location(index, file_id, reference.clone())?;
    Some(location_to_link(text, reference.range, location))
}

fn location_to_link(text: &Rope, origin_range: TextRange, location: Location) -> LocationLink {
    LocationLink {
        origin_selection_range: Some(range_to_lsp_range(text, origin_range)),
        target_uri: location.uri,
        target_range: location.range,
        target_selection_range: location.range,
    }
}

fn dependency_target_location(
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

fn reference_target_location(
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

fn class_location(index: &mut EchoIndex, path: &Path, class_name: &str) -> Option<Location> {
    let class_file_id = index_php_file(index, path)?;
    if let Some(location) = class_symbol_location(index, class_file_id, class_name) {
        return symbol_location(index, location.file_id, location.selection_range);
    }
    if let Some(range) = php_class_name_range(path, class_name) {
        return file_location(index, path, range);
    }
    file_location(index, path, TextRange::new(0, 0))
}

pub fn receiver_method_definition_at(
    index: &mut EchoIndex,
    file_id: FileId,
    offset: TextOffset,
    symbols: &[Symbol],
    dependencies: &[DependencyFact],
) -> Option<DefinitionLocation> {
    let reference = index
        .references(ReferenceQuery::at(file_id, offset))
        .into_iter()
        .find(|reference| reference.kind == ReferenceKind::Method)?
        .clone();
    let receiver = reference.qualifier.as_deref()?;
    let receiver_type = local_variable_type(symbols, receiver)?;
    let class_name = resolve_imported_type(dependencies, &receiver_type).unwrap_or(receiver_type);
    if index
        .method_definition(&class_name, &reference.name)
        .is_none()
    {
        if let Some(path) = composer_class_file(index, &class_name) {
            index_php_file(index, &path);
        }
    }
    index
        .method_definition(&class_name, &reference.name)
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
        DefinitionLocation::File {
            file_id,
            selection_range,
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

fn file_location(index: &mut EchoIndex, path: &Path, range: TextRange) -> Option<Location> {
    let path = std::fs::canonicalize(path).ok()?;
    index_php_file(index, &path);
    let file_id = ensure_index_file(index, &path);
    symbol_location(index, file_id, range)
}

fn symbol_location(index: &EchoIndex, file_id: FileId, range: TextRange) -> Option<Location> {
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

fn index_php_file(index: &mut EchoIndex, path: &Path) -> Option<FileId> {
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

fn resolve_imported_class_name(index: &EchoIndex, file_id: FileId, name: &str) -> Option<String> {
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

fn local_variable_type(symbols: &[Symbol], name: &str) -> Option<String> {
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

fn resolve_imported_type(dependencies: &[DependencyFact], ty: &str) -> Option<String> {
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
        DefinitionLocation, DependencyFact, DependencyKind, EchoFileMode, EchoIndex, FileId,
        FqName, IndexFacts, IndexedFile, ReferenceFact, ReferenceKind, Signature, SymbolFact,
        SymbolKind, SymbolName, TextRange,
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
            target_range: TextRange::new(6, 43),
        };

        let definition = method_definition_at(
            &index,
            &Rope::from_str("<?php\n$app->handleRequest(Request::capture());\n"),
            TextOffset(13),
            &[app],
            &[dependency],
        )
        .expect("method definition");

        let DefinitionLocation::Symbol(location) = definition else {
            panic!("expected symbol definition");
        };
        assert_eq!(location.file_id, vendor_file_id);
        assert_eq!(location.selection_range, TextRange::new(76, 89));
    }

    #[test]
    fn resolves_require_dependency_to_target_file() {
        let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tests/php/112_laravel_bootstrap")
            .canonicalize()
            .expect("fixture root");
        let autoload = fixture_root.join("vendor/autoload.php");
        let mut index = EchoIndex::new();
        let file_id = FileId(1);
        index.insert_file(IndexedFile {
            file_id,
            uri: "file:///project/public/index.php".to_string(),
            path: None,
            version: None,
            mode: EchoFileMode::PhpCompat,
            content_hash: None,
        });
        index.update_file(
            file_id,
            IndexFacts {
                file_id,
                mode: EchoFileMode::PhpCompat,
                declarations: Vec::new(),
                dependencies: vec![DependencyFact {
                    kind: DependencyKind::ComposerAutoload,
                    target: autoload.to_string_lossy().to_string(),
                    alias: None,
                    range: TextRange::new(10, 55),
                    target_range: TextRange::new(18, 54),
                }],
                references: Vec::new(),
            },
        );

        let location = dependency_target_location_at(&mut index, file_id, TextOffset(25))
            .expect("autoload target location");

        assert_eq!(location.uri, Uri::from_file_path(&autoload).unwrap());
    }

    #[test]
    fn resolves_php_use_dependency_through_composer_psr4() {
        let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tests/php/112_laravel_bootstrap")
            .canonicalize()
            .expect("fixture root");
        let autoload = fixture_root.join("vendor/autoload.php");
        let request = fixture_root.join("vendor/laravel/framework/src/Illuminate/Http/Request.php");
        let request_source = std::fs::read_to_string(&request).expect("request source");
        let class_start = request_source.find("Request").expect("request class");
        let mut index = EchoIndex::new();
        let file_id = FileId(1);
        index.insert_file(IndexedFile {
            file_id,
            uri: "file:///project/public/index.php".to_string(),
            path: None,
            version: None,
            mode: EchoFileMode::PhpCompat,
            content_hash: None,
        });
        index.update_file(
            file_id,
            IndexFacts {
                file_id,
                mode: EchoFileMode::PhpCompat,
                declarations: Vec::new(),
                dependencies: vec![
                    DependencyFact {
                        kind: DependencyKind::ComposerAutoload,
                        target: autoload.to_string_lossy().to_string(),
                        alias: None,
                        range: TextRange::new(50, 90),
                        target_range: TextRange::new(58, 89),
                    },
                    DependencyFact {
                        kind: DependencyKind::PhpUse,
                        target: "Illuminate\\Http\\Request".to_string(),
                        alias: None,
                        range: TextRange::new(0, 30),
                        target_range: TextRange::new(4, 29),
                    },
                ],
                references: Vec::new(),
            },
        );

        let location = dependency_target_location_at(&mut index, file_id, TextOffset(10))
            .expect("Request target location");

        assert_eq!(location.uri, Uri::from_file_path(&request).unwrap());
        assert_eq!(
            location.range.start,
            range_to_lsp_range(
                &Rope::from_str(&request_source),
                TextRange::new(class_start as u32, class_start as u32)
            )
            .start
        );
    }

    #[test]
    fn resolves_class_reference_through_composer_psr4() {
        let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tests/php/112_laravel_bootstrap")
            .canonicalize()
            .expect("fixture root");
        let autoload = fixture_root.join("vendor/autoload.php");
        let request = fixture_root.join("vendor/laravel/framework/src/Illuminate/Http/Request.php");
        let request_source = std::fs::read_to_string(&request).expect("request source");
        let class_start = request_source.find("Request").expect("request class");
        let mut index = EchoIndex::new();
        let file_id = FileId(1);
        index.insert_file(IndexedFile {
            file_id,
            uri: "file:///project/public/index.php".to_string(),
            path: None,
            version: None,
            mode: EchoFileMode::PhpCompat,
            content_hash: None,
        });
        index.update_file(
            file_id,
            IndexFacts {
                file_id,
                mode: EchoFileMode::PhpCompat,
                declarations: Vec::new(),
                dependencies: vec![
                    DependencyFact {
                        kind: DependencyKind::ComposerAutoload,
                        target: autoload.to_string_lossy().to_string(),
                        alias: None,
                        range: TextRange::new(0, 20),
                        target_range: TextRange::new(0, 20),
                    },
                    DependencyFact {
                        kind: DependencyKind::PhpUse,
                        target: "Illuminate\\Http\\Request".to_string(),
                        alias: None,
                        range: TextRange::new(30, 60),
                        target_range: TextRange::new(34, 58),
                    },
                ],
                references: vec![ReferenceFact {
                    kind: ReferenceKind::ClassLike,
                    name: "Request".to_string(),
                    qualifier: None,
                    range: TextRange::new(80, 87),
                }],
            },
        );

        let location = reference_target_location_at(&mut index, file_id, TextOffset(82))
            .expect("Request class location");

        assert_eq!(location.uri, Uri::from_file_path(&request).unwrap());
        assert_eq!(
            location.range.start,
            range_to_lsp_range(
                &Rope::from_str(&request_source),
                TextRange::new(class_start as u32, class_start as u32)
            )
            .start
        );
    }

    #[test]
    fn resolves_class_reference_to_source_line_when_target_php_is_not_parseable() {
        let root = std::env::temp_dir().join(format!("echo-lsp-class-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let composer_dir = root.join("vendor/composer");
        let request_dir = root.join("vendor/laravel/framework/src/Illuminate/Http");
        std::fs::create_dir_all(&composer_dir).expect("composer dir");
        std::fs::create_dir_all(&request_dir).expect("request dir");
        let autoload = root.join("vendor/autoload.php");
        let request = request_dir.join("Request.php");
        std::fs::write(&autoload, "<?php\n").expect("autoload");
        std::fs::write(
            composer_dir.join("autoload_psr4.php"),
            "<?php\n$vendorDir = dirname(__DIR__);\n$baseDir = dirname($vendorDir);\nreturn array(\n    'Illuminate\\\\' => array($vendorDir . '/laravel/framework/src/Illuminate'),\n);\n",
        )
        .expect("autoload psr4");
        std::fs::write(
            &request,
            "<?php\nnamespace Illuminate\\Http;\nfinal class Request\n{\n    use Macroable;\n}\n",
        )
        .expect("request source");

        let mut index = EchoIndex::new();
        let file_id = FileId(1);
        index.insert_file(IndexedFile {
            file_id,
            uri: "file:///project/public/index.php".to_string(),
            path: None,
            version: None,
            mode: EchoFileMode::PhpCompat,
            content_hash: None,
        });
        index.update_file(
            file_id,
            IndexFacts {
                file_id,
                mode: EchoFileMode::PhpCompat,
                declarations: Vec::new(),
                dependencies: vec![
                    DependencyFact {
                        kind: DependencyKind::ComposerAutoload,
                        target: autoload.to_string_lossy().to_string(),
                        alias: None,
                        range: TextRange::new(0, 20),
                        target_range: TextRange::new(0, 20),
                    },
                    DependencyFact {
                        kind: DependencyKind::PhpUse,
                        target: "Illuminate\\Http\\Request".to_string(),
                        alias: None,
                        range: TextRange::new(30, 60),
                        target_range: TextRange::new(34, 58),
                    },
                ],
                references: vec![ReferenceFact {
                    kind: ReferenceKind::ClassLike,
                    name: "Request".to_string(),
                    qualifier: None,
                    range: TextRange::new(80, 87),
                }],
            },
        );

        let location = reference_target_location_at(&mut index, file_id, TextOffset(82))
            .expect("Request class location");

        assert_eq!(location.uri, Uri::from_file_path(&request).unwrap());
        assert_eq!(location.range.start.line, 2);
        assert_eq!(location.range.start.character, 12);

        std::fs::remove_dir_all(&root).expect("cleanup");
    }

    #[test]
    fn resolves_static_method_reference_to_composer_class_declaration() {
        let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tests/php/112_laravel_bootstrap")
            .canonicalize()
            .expect("fixture root");
        let autoload = fixture_root.join("vendor/autoload.php");
        let request = fixture_root.join("vendor/laravel/framework/src/Illuminate/Http/Request.php");
        let request_source = std::fs::read_to_string(&request).expect("request source");
        let capture_start = request_source.find("capture").expect("capture method");
        let mut index = EchoIndex::new();
        let file_id = FileId(1);
        index.insert_file(IndexedFile {
            file_id,
            uri: "file:///project/public/index.php".to_string(),
            path: None,
            version: None,
            mode: EchoFileMode::PhpCompat,
            content_hash: None,
        });
        index.update_file(
            file_id,
            IndexFacts {
                file_id,
                mode: EchoFileMode::PhpCompat,
                declarations: Vec::new(),
                dependencies: vec![
                    DependencyFact {
                        kind: DependencyKind::ComposerAutoload,
                        target: autoload.to_string_lossy().to_string(),
                        alias: None,
                        range: TextRange::new(0, 20),
                        target_range: TextRange::new(0, 20),
                    },
                    DependencyFact {
                        kind: DependencyKind::PhpUse,
                        target: "Illuminate\\Http\\Request".to_string(),
                        alias: None,
                        range: TextRange::new(30, 60),
                        target_range: TextRange::new(34, 58),
                    },
                ],
                references: vec![ReferenceFact {
                    kind: ReferenceKind::StaticMethod,
                    name: "capture".to_string(),
                    qualifier: Some("Request".to_string()),
                    range: TextRange::new(89, 96),
                }],
            },
        );

        let location = reference_target_location_at(&mut index, file_id, TextOffset(90))
            .expect("capture method location");

        assert_eq!(location.uri, Uri::from_file_path(&request).unwrap());
        assert_eq!(
            location.range.start,
            range_to_lsp_range(
                &Rope::from_str(&request_source),
                TextRange::new(capture_start as u32, capture_start as u32)
            )
            .start
        );
    }

    #[test]
    fn resolves_static_method_reference_to_source_line_when_target_php_is_not_parseable() {
        let root =
            std::env::temp_dir().join(format!("echo-lsp-static-method-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let composer_dir = root.join("vendor/composer");
        let request_dir = root.join("vendor/laravel/framework/src/Illuminate/Http");
        std::fs::create_dir_all(&composer_dir).expect("composer dir");
        std::fs::create_dir_all(&request_dir).expect("request dir");
        let autoload = root.join("vendor/autoload.php");
        let request = request_dir.join("Request.php");
        std::fs::write(&autoload, "<?php\n").expect("autoload");
        std::fs::write(
            composer_dir.join("autoload_psr4.php"),
            "<?php\n$vendorDir = dirname(__DIR__);\n$baseDir = dirname($vendorDir);\nreturn array(\n    'Illuminate\\\\' => array($vendorDir . '/laravel/framework/src/Illuminate'),\n);\n",
        )
        .expect("autoload psr4");
        let request_source = "<?php\nnamespace Illuminate\\Http;\nclass Request\n{\n    use Macroable;\n\n    public static function capture()\n    {\n    }\n}\n";
        std::fs::write(&request, request_source).expect("request source");

        let mut index = EchoIndex::new();
        let file_id = FileId(1);
        index.insert_file(IndexedFile {
            file_id,
            uri: "file:///project/public/index.php".to_string(),
            path: None,
            version: None,
            mode: EchoFileMode::PhpCompat,
            content_hash: None,
        });
        index.update_file(
            file_id,
            IndexFacts {
                file_id,
                mode: EchoFileMode::PhpCompat,
                declarations: Vec::new(),
                dependencies: vec![
                    DependencyFact {
                        kind: DependencyKind::ComposerAutoload,
                        target: autoload.to_string_lossy().to_string(),
                        alias: None,
                        range: TextRange::new(0, 20),
                        target_range: TextRange::new(0, 20),
                    },
                    DependencyFact {
                        kind: DependencyKind::PhpUse,
                        target: "Illuminate\\Http\\Request".to_string(),
                        alias: None,
                        range: TextRange::new(30, 60),
                        target_range: TextRange::new(34, 58),
                    },
                ],
                references: vec![ReferenceFact {
                    kind: ReferenceKind::StaticMethod,
                    name: "capture".to_string(),
                    qualifier: Some("Request".to_string()),
                    range: TextRange::new(89, 96),
                }],
            },
        );

        let location = reference_target_location_at(&mut index, file_id, TextOffset(90))
            .expect("capture method location");

        assert_eq!(location.uri, Uri::from_file_path(&request).unwrap());
        assert_eq!(location.range.start.line, 6);
        assert_eq!(location.range.start.character, 27);

        std::fs::remove_dir_all(&root).expect("cleanup");
    }

    #[test]
    fn resolves_phpdoc_receiver_method_through_composer_psr4() {
        let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tests/php/112_laravel_bootstrap")
            .canonicalize()
            .expect("fixture root");
        let autoload = fixture_root.join("vendor/autoload.php");
        let application =
            fixture_root.join("vendor/laravel/framework/src/Illuminate/Foundation/Application.php");
        let mut index = EchoIndex::new();
        let file_id = FileId(1);
        index.insert_file(IndexedFile {
            file_id,
            uri: "file:///project/public/index.php".to_string(),
            path: None,
            version: None,
            mode: EchoFileMode::PhpCompat,
            content_hash: None,
        });
        index.update_file(
            file_id,
            IndexFacts {
                file_id,
                mode: EchoFileMode::PhpCompat,
                declarations: Vec::new(),
                dependencies: vec![
                    DependencyFact {
                        kind: DependencyKind::ComposerAutoload,
                        target: autoload.to_string_lossy().to_string(),
                        alias: None,
                        range: TextRange::new(0, 20),
                        target_range: TextRange::new(0, 20),
                    },
                    DependencyFact {
                        kind: DependencyKind::PhpUse,
                        target: "Illuminate\\Foundation\\Application".to_string(),
                        alias: None,
                        range: TextRange::new(30, 75),
                        target_range: TextRange::new(34, 70),
                    },
                ],
                references: vec![ReferenceFact {
                    kind: ReferenceKind::Method,
                    name: "handleRequest".to_string(),
                    qualifier: Some("app".to_string()),
                    range: TextRange::new(90, 103),
                }],
            },
        );
        let app = echo_index::Symbol {
            id: echo_index::SymbolId(1),
            file_id,
            name: SymbolName::new("app"),
            fq_name: None,
            kind: SymbolKind::LocalVariable,
            range: TextRange::new(80, 84),
            selection_range: TextRange::new(80, 84),
            visibility: None,
            container: None,
            signature: Some(Signature {
                text: "Application".to_string(),
            }),
        };
        let dependencies = index
            .dependencies(DependencyQuery::in_file(file_id))
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();

        let definition = receiver_method_definition_at(
            &mut index,
            file_id,
            TextOffset(95),
            &[app],
            &dependencies,
        )
        .expect("handleRequest definition");

        let DefinitionLocation::Symbol(location) = definition else {
            panic!("expected symbol location");
        };
        let file = index.file(location.file_id).expect("application file");
        assert_eq!(file.path.as_deref(), Some(application.as_path()));
    }

    #[test]
    fn resolves_file_path_reference_to_target_file() {
        let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tests/php/112_laravel_bootstrap")
            .canonicalize()
            .expect("fixture root");
        let autoload = fixture_root.join("vendor/autoload.php");
        let mut index = EchoIndex::new();
        let file_id = FileId(1);
        index.insert_file(IndexedFile {
            file_id,
            uri: "file:///project/public/index.php".to_string(),
            path: None,
            version: None,
            mode: EchoFileMode::PhpCompat,
            content_hash: None,
        });
        index.update_file(
            file_id,
            IndexFacts {
                file_id,
                mode: EchoFileMode::PhpCompat,
                declarations: Vec::new(),
                dependencies: Vec::new(),
                references: vec![ReferenceFact {
                    kind: ReferenceKind::FilePath,
                    name: autoload.to_string_lossy().to_string(),
                    qualifier: None,
                    range: TextRange::new(10, 45),
                }],
            },
        );

        let location = reference_target_location_at(&mut index, file_id, TextOffset(20))
            .expect("autoload path location");

        assert_eq!(location.uri, Uri::from_file_path(&autoload).unwrap());
    }

    #[test]
    fn file_path_reference_link_uses_full_origin_expression() {
        let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tests/php/112_laravel_bootstrap")
            .canonicalize()
            .expect("fixture root");
        let autoload = fixture_root.join("vendor/autoload.php");
        let source = "<?php\nrequire __DIR__.'/../vendor/autoload.php';\n";
        let expr = "__DIR__.'/../vendor/autoload.php'";
        let start = source.find(expr).expect("expr");
        let mut index = EchoIndex::new();
        let file_id = FileId(1);
        index.insert_file(IndexedFile {
            file_id,
            uri: "file:///project/public/index.php".to_string(),
            path: None,
            version: None,
            mode: EchoFileMode::PhpCompat,
            content_hash: None,
        });
        index.update_file(
            file_id,
            IndexFacts {
                file_id,
                mode: EchoFileMode::PhpCompat,
                declarations: Vec::new(),
                dependencies: Vec::new(),
                references: vec![ReferenceFact {
                    kind: ReferenceKind::FilePath,
                    name: autoload.to_string_lossy().to_string(),
                    qualifier: None,
                    range: TextRange::new(start as u32, (start + expr.len()) as u32),
                }],
            },
        );

        let link = reference_target_link_at(
            &mut index,
            &Rope::from_str(source),
            file_id,
            TextOffset((start + expr.find("vendor").expect("vendor")) as u32),
        )
        .expect("autoload link");

        assert_eq!(link.target_uri, Uri::from_file_path(&autoload).unwrap());
        assert_eq!(
            link.origin_selection_range,
            Some(range_to_lsp_range(
                &Rope::from_str(source),
                TextRange::new(start as u32, (start + expr.len()) as u32),
            ))
        );
    }

    #[test]
    fn file_path_reference_link_skips_missing_dir_target() {
        let target = "/project/public/../storage/framework/maintenance.php";
        let source = "<?php\nfile_exists(__DIR__.'/../storage/framework/maintenance.php');\n";
        let expr = "__DIR__.'/../storage/framework/maintenance.php'";
        let start = source.find(expr).expect("expr");
        let mut index = EchoIndex::new();
        let file_id = FileId(1);
        index.insert_file(IndexedFile {
            file_id,
            uri: "file:///project/public/index.php".to_string(),
            path: None,
            version: None,
            mode: EchoFileMode::PhpCompat,
            content_hash: None,
        });
        index.update_file(
            file_id,
            IndexFacts {
                file_id,
                mode: EchoFileMode::PhpCompat,
                declarations: Vec::new(),
                dependencies: Vec::new(),
                references: vec![ReferenceFact {
                    kind: ReferenceKind::FilePath,
                    name: target.to_string(),
                    qualifier: None,
                    range: TextRange::new(start as u32, (start + expr.len()) as u32),
                }],
            },
        );

        let link = reference_target_link_at(
            &mut index,
            &Rope::from_str(source),
            file_id,
            TextOffset((start + expr.find("storage").expect("storage")) as u32),
        );

        assert!(link.is_none());
    }
}
