use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceSpan {
    pub source_id: SourceId,
    pub span: Span,
}

impl SourceSpan {
    pub const fn new(source_id: SourceId, span: Span) -> Self {
        Self { source_id, span }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SourceId(u32);

impl SourceId {
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u32 {
        self.0
    }

    fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    File,
    Repl,
    Std,
    Anonymous,
}

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub id: Option<SourceId>,
    pub path: PathBuf,
    pub text: String,
    pub kind: SourceKind,
}

impl SourceFile {
    pub fn new(path: PathBuf, text: String) -> Self {
        Self {
            id: None,
            path,
            text,
            kind: SourceKind::File,
        }
    }

    pub fn repl(text: String) -> Self {
        Self {
            id: None,
            path: PathBuf::from("<repl>"),
            text,
            kind: SourceKind::Repl,
        }
    }

    pub fn std(path: PathBuf, text: String) -> Self {
        Self {
            id: None,
            path,
            text,
            kind: SourceKind::Std,
        }
    }

    pub fn anonymous(text: String) -> Self {
        Self {
            id: None,
            path: PathBuf::from("<anonymous>"),
            text,
            kind: SourceKind::Anonymous,
        }
    }

    pub fn with_id(mut self, id: SourceId) -> Self {
        self.id = Some(id);
        self
    }
}

#[derive(Debug, Default)]
pub struct SourceMap {
    files: Vec<SourceFile>,
    ids_by_path: HashMap<PathBuf, SourceId>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, file: SourceFile) -> SourceId {
        let id = SourceId::new(self.files.len() as u32);
        let file = file.with_id(id);
        self.ids_by_path.insert(file.path.clone(), id);
        self.files.push(file);
        id
    }

    pub fn insert_path(&mut self, path: PathBuf, text: String) -> SourceId {
        self.insert(SourceFile::new(path, text))
    }

    pub fn get(&self, id: SourceId) -> Option<&SourceFile> {
        self.files.get(id.index())
    }

    pub fn get_by_path(&self, path: impl AsRef<Path>) -> Option<&SourceFile> {
        self.id_for_path(path).and_then(|id| self.get(id))
    }

    pub fn id_for_path(&self, path: impl AsRef<Path>) -> Option<SourceId> {
        self.ids_by_path.get(path.as_ref()).copied()
    }

    pub fn iter(&self) -> impl Iterator<Item = (SourceId, &SourceFile)> {
        self.files
            .iter()
            .enumerate()
            .map(|(index, file)| (SourceId::new(index as u32), file))
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_map_assigns_stable_ids() {
        let mut sources = SourceMap::new();

        let first = sources.insert_path(PathBuf::from("app.php"), "<?php echo 1;".to_string());
        let second = sources.insert_path(PathBuf::from("app.echo"), "echo 2".to_string());

        assert_eq!(first.raw(), 0);
        assert_eq!(second.raw(), 1);
        assert_eq!(sources.get(first).unwrap().id, Some(first));
        assert_eq!(sources.get(second).unwrap().path, PathBuf::from("app.echo"));
        assert_eq!(sources.len(), 2);
    }

    #[test]
    fn source_map_looks_up_latest_file_for_path() {
        let mut sources = SourceMap::new();
        let old = sources.insert_path(PathBuf::from("app.php"), "old".to_string());
        let new = sources.insert_path(PathBuf::from("app.php"), "new".to_string());

        assert_eq!(sources.get(old).unwrap().text, "old");
        assert_eq!(sources.id_for_path("app.php"), Some(new));
        assert_eq!(sources.get_by_path("app.php").unwrap().text, "new");
    }

    #[test]
    fn source_file_constructors_record_kind_without_assigning_id() {
        assert_eq!(
            SourceFile::repl("echo 1".to_string()).kind,
            SourceKind::Repl
        );
        assert_eq!(
            SourceFile::std(
                PathBuf::from("std/time.echo"),
                "module std.time".to_string()
            )
            .kind,
            SourceKind::Std
        );
        assert_eq!(
            SourceFile::anonymous("echo 1".to_string()).kind,
            SourceKind::Anonymous
        );
        assert_eq!(
            SourceFile::new(PathBuf::from("app.php"), "<?php".to_string()).id,
            None
        );
    }
}
