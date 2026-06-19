use crate::{DependencyFact, FileId, SymbolId, TextRange};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymbolLocation {
    pub file_id: FileId,
    pub symbol_id: SymbolId,
    pub range: TextRange,
    pub selection_range: TextRange,
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
