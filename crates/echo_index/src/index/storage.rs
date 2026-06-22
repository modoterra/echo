use crate::{EchoIndex, FileId, IndexFacts, IndexedFile, Symbol, SymbolId};

impl EchoIndex {
    pub fn insert_file(&mut self, file: IndexedFile) {
        self.files.insert(file.file_id, file);
    }

    pub fn file(&self, file_id: FileId) -> Option<&IndexedFile> {
        self.files.get(&file_id)
    }

    pub fn file_by_path(&self, path: &std::path::Path) -> Option<&IndexedFile> {
        self.files
            .values()
            .find(|file| file.path.as_deref() == Some(path))
    }

    pub fn file_id_by_path(&self, path: &std::path::Path) -> Option<FileId> {
        self.file_by_path(path).map(|file| file.file_id)
    }

    pub fn remove_file(&mut self, file_id: FileId) {
        self.files.remove(&file_id);
        self.remove_symbols_for_file(file_id);
        self.dependencies_by_file.remove(&file_id);
        self.references_by_file.remove(&file_id);
    }

    pub fn update_file(&mut self, file_id: FileId, facts: IndexFacts) {
        debug_assert_eq!(file_id, facts.file_id);
        self.remove_symbols_for_file(file_id);
        self.dependencies_by_file
            .insert(file_id, facts.dependencies);
        self.references_by_file.insert(file_id, facts.references);

        let mut symbol_ids = Vec::with_capacity(facts.declarations.len());
        for fact in facts.declarations {
            let symbol_id = self.alloc_symbol_id();
            let name_key = fact.name.text.to_string();
            let fq_name_key = fact.fq_name.clone();
            let symbol = Symbol {
                id: symbol_id,
                file_id,
                name: fact.name,
                fq_name: fact.fq_name,
                kind: fact.kind,
                range: fact.range,
                selection_range: fact.selection_range,
                visibility: fact.visibility,
                container: None,
                signature: fact.signature,
            };

            self.symbols.insert(symbol_id, symbol);
            self.symbols_by_name
                .entry(name_key)
                .or_default()
                .push(symbol_id);
            if let Some(fq_name) = fq_name_key {
                self.symbols_by_fq_name
                    .entry(fq_name)
                    .or_default()
                    .push(symbol_id);
            }
            symbol_ids.push(symbol_id);
        }

        self.symbols_by_file.insert(file_id, symbol_ids);
    }

    fn alloc_symbol_id(&mut self) -> SymbolId {
        let symbol_id = SymbolId(self.next_symbol_id);
        self.next_symbol_id += 1;
        symbol_id
    }

    fn remove_symbols_for_file(&mut self, file_id: FileId) {
        let Some(symbol_ids) = self.symbols_by_file.remove(&file_id) else {
            return;
        };

        for symbol_id in &symbol_ids {
            self.symbols.remove(symbol_id);
        }

        for symbol_ids_by_name in self.symbols_by_name.values_mut() {
            symbol_ids_by_name.retain(|symbol_id| !symbol_ids.contains(symbol_id));
        }
        self.symbols_by_name
            .retain(|_, symbol_ids| !symbol_ids.is_empty());

        for symbol_ids_by_fq_name in self.symbols_by_fq_name.values_mut() {
            symbol_ids_by_fq_name.retain(|symbol_id| !symbol_ids.contains(symbol_id));
        }
        self.symbols_by_fq_name
            .retain(|_, symbol_ids| !symbol_ids.is_empty());
    }
}
