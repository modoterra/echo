use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FileId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextRange {
    pub start: u32,
    pub end: u32,
}

impl TextRange {
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    pub const fn contains(self, offset: TextOffset) -> bool {
        self.start <= offset.0 && offset.0 < self.end
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextOffset(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EchoFileMode {
    Echo,
    PhpCompat,
}

#[derive(Debug, Clone)]
pub struct IndexedFile {
    pub file_id: FileId,
    pub uri: String,
    pub path: Option<PathBuf>,
    pub version: Option<i32>,
    pub mode: EchoFileMode,
    pub content_hash: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyFact {
    pub kind: DependencyKind,
    pub target: String,
    pub alias: Option<String>,
    pub range: TextRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceFact {
    pub kind: ReferenceKind,
    pub name: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReferenceKind {
    ClassLike,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DependencyKind {
    PhpUse,
    EchoStdImport,
    EchoFileImport,
    Require,
    RequireOnce,
    Include,
    IncludeOnce,
    ComposerAutoload,
}
