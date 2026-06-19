use crate::{DependencyFact, FileId, ReferenceFact, SymbolId, TextOffset, TextRange};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymbolLocation {
    pub file_id: FileId,
    pub symbol_id: SymbolId,
    pub range: TextRange,
    pub selection_range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReferenceLocation {
    pub file_id: FileId,
    pub range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefinitionLocation {
    Symbol(SymbolLocation),
    Dependency {
        file_id: FileId,
        range: TextRange,
        selection_range: TextRange,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyQuery<'a> {
    pub file_id: Option<FileId>,
    pub target_contains: Option<&'a str>,
}

impl<'a> DependencyQuery<'a> {
    pub const fn all() -> Self {
        Self {
            file_id: None,
            target_contains: None,
        }
    }

    pub const fn in_file(file_id: FileId) -> Self {
        Self {
            file_id: Some(file_id),
            target_contains: None,
        }
    }

    pub fn matches(&self, file_id: FileId, dependency: &DependencyFact) -> bool {
        if self.file_id.is_some_and(|query_file| query_file != file_id) {
            return false;
        }

        if let Some(query) = self.target_contains {
            dependency.target.contains(query)
        } else {
            true
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceQuery {
    pub file_id: Option<FileId>,
    pub offset: Option<TextOffset>,
}

impl ReferenceQuery {
    pub const fn in_file(file_id: FileId) -> Self {
        Self {
            file_id: Some(file_id),
            offset: None,
        }
    }

    pub const fn at(file_id: FileId, offset: TextOffset) -> Self {
        Self {
            file_id: Some(file_id),
            offset: Some(offset),
        }
    }

    pub fn matches(&self, file_id: FileId, reference: &ReferenceFact) -> bool {
        if self.file_id.is_some_and(|query_file| query_file != file_id) {
            return false;
        }

        if let Some(offset) = self.offset {
            reference.range.contains(offset)
        } else {
            true
        }
    }
}
