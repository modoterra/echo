use std::collections::HashMap;

mod lookup;
mod storage;

use crate::{
    DependencyFact, FileId, FqName, IndexedFile, ReferenceFact, Symbol, SymbolFact, SymbolId,
};

#[derive(Debug, Clone)]
pub struct IndexFacts {
    pub file_id: FileId,
    pub declarations: Vec<SymbolFact>,
    pub dependencies: Vec<DependencyFact>,
    pub references: Vec<ReferenceFact>,
}

impl IndexFacts {
    pub fn declarations(file_id: FileId, declarations: Vec<SymbolFact>) -> Self {
        Self {
            file_id,
            declarations,
            dependencies: Vec::new(),
            references: Vec::new(),
        }
    }
}

#[derive(Debug, Default)]
pub struct EchoIndex {
    next_file_id: u32,
    next_symbol_id: u64,
    files: HashMap<FileId, IndexedFile>,
    symbols: HashMap<SymbolId, Symbol>,
    symbols_by_file: HashMap<FileId, Vec<SymbolId>>,
    symbols_by_name: HashMap<String, Vec<SymbolId>>,
    symbols_by_fq_name: HashMap<FqName, Vec<SymbolId>>,
    dependencies_by_file: HashMap<FileId, Vec<DependencyFact>>,
    references_by_file: HashMap<FileId, Vec<ReferenceFact>>,
}

impl Clone for EchoIndex {
    fn clone(&self) -> Self {
        Self {
            next_file_id: self.next_file_id,
            next_symbol_id: self.next_symbol_id,
            files: self.files.clone(),
            symbols: self.symbols.clone(),
            symbols_by_file: self.symbols_by_file.clone(),
            symbols_by_name: self.symbols_by_name.clone(),
            symbols_by_fq_name: self.symbols_by_fq_name.clone(),
            dependencies_by_file: self.dependencies_by_file.clone(),
            references_by_file: self.references_by_file.clone(),
        }
    }
}

impl EchoIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn alloc_file_id(&mut self) -> FileId {
        let file_id = FileId(self.next_file_id);
        self.next_file_id += 1;
        file_id
    }

    pub fn document_symbols(&self, file_id: FileId) -> Vec<&Symbol> {
        self.symbols_by_file
            .get(&file_id)
            .into_iter()
            .flatten()
            .filter_map(|symbol_id| self.symbols.get(symbol_id))
            .collect()
    }

    pub fn workspace_symbols(&self, query: &str, limit: usize) -> Vec<&Symbol> {
        if limit == 0 {
            return Vec::new();
        }

        let query = query.to_ascii_lowercase();
        let mut symbols = Vec::new();

        for (name, symbol_ids) in &self.symbols_by_name {
            if !query.is_empty() && !name.to_ascii_lowercase().contains(&query) {
                continue;
            }

            for symbol_id in symbol_ids {
                if let Some(symbol) = self.symbols.get(symbol_id) {
                    symbols.push(symbol);
                    if symbols.len() == limit {
                        return symbols;
                    }
                }
            }
        }

        symbols
    }

    pub fn symbol(&self, symbol_id: SymbolId) -> Option<&Symbol> {
        self.symbols.get(&symbol_id)
    }

    pub fn symbols_by_fq_name(&self, fq_name: &FqName) -> Vec<&Symbol> {
        self.symbols_by_fq_name
            .get(fq_name)
            .into_iter()
            .flatten()
            .filter_map(|symbol_id| self.symbols.get(symbol_id))
            .collect()
    }
}

#[cfg(test)]
#[path = "index/tests.rs"]
mod tests;
