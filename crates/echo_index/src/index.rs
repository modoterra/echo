use std::collections::HashMap;

use crate::{
    DefinitionLocation, DependencyFact, DependencyQuery, EchoFileMode, FileId, FqName, IndexedFile,
    ReferenceFact, ReferenceLocation, ReferenceQuery, Symbol, SymbolFact, SymbolId, SymbolLocation,
    TextOffset,
};

#[derive(Debug, Clone)]
pub struct IndexFacts {
    pub file_id: FileId,
    pub mode: EchoFileMode,
    pub declarations: Vec<SymbolFact>,
    pub dependencies: Vec<DependencyFact>,
    pub references: Vec<ReferenceFact>,
}

impl IndexFacts {
    pub fn declarations(
        file_id: FileId,
        mode: EchoFileMode,
        declarations: Vec<SymbolFact>,
    ) -> Self {
        Self {
            file_id,
            mode,
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

    pub fn method_definition(&self, class_name: &str, method_name: &str) -> Option<SymbolLocation> {
        let target = format!("{class_name}::{method_name}");
        let short_target = class_name
            .rsplit('\\')
            .next()
            .map(|short_name| format!("{short_name}::{method_name}"));

        self.symbols
            .values()
            .find(|symbol| {
                symbol.kind == crate::SymbolKind::Method
                    && symbol.fq_name.as_ref().is_some_and(|fq_name| {
                        let fq_name = fq_name.as_string();
                        fq_name == target
                            || short_target
                                .as_ref()
                                .is_some_and(|short_target| fq_name == *short_target)
                    })
            })
            .map(|symbol| SymbolLocation {
                file_id: symbol.file_id,
                symbol_id: symbol.id,
                range: symbol.range,
                selection_range: symbol.selection_range,
            })
    }

    pub fn dependencies(&self, query: DependencyQuery<'_>) -> Vec<&DependencyFact> {
        let mut matches = Vec::new();

        for (file_id, dependencies) in &self.dependencies_by_file {
            if query
                .file_id
                .is_some_and(|query_file| query_file != *file_id)
            {
                continue;
            }

            matches.extend(
                dependencies
                    .iter()
                    .filter(|dependency| query.matches(*file_id, dependency)),
            );
        }

        matches
    }

    pub fn references(&self, query: ReferenceQuery) -> Vec<&ReferenceFact> {
        let mut matches = Vec::new();

        for (file_id, references) in &self.references_by_file {
            if query
                .file_id
                .is_some_and(|query_file| query_file != *file_id)
            {
                continue;
            }

            matches.extend(
                references
                    .iter()
                    .filter(|reference| query.matches(*file_id, reference)),
            );
        }

        matches
    }

    pub fn definition_at(&self, file_id: FileId, offset: TextOffset) -> Option<DefinitionLocation> {
        for symbol in self.document_symbols(file_id) {
            if symbol.selection_range.contains(offset) || symbol.range.contains(offset) {
                return Some(DefinitionLocation::Symbol(SymbolLocation {
                    file_id,
                    symbol_id: symbol.id,
                    range: symbol.range,
                    selection_range: symbol.selection_range,
                }));
            }
        }

        let dependencies = self.dependencies(DependencyQuery::in_file(file_id));
        for dependency in &dependencies {
            if dependency.range.contains(offset) {
                return Some(DefinitionLocation::Dependency {
                    file_id,
                    range: dependency.range,
                    selection_range: dependency.range,
                });
            }
        }

        let reference = self
            .references(ReferenceQuery::at(file_id, offset))
            .into_iter()
            .next()?;
        let dependency = dependencies.into_iter().find(|dependency| {
            dependency.alias.as_deref() == Some(reference.name.as_str())
                || dependency
                    .target
                    .rsplit('\\')
                    .next()
                    .is_some_and(|name| name == reference.name)
        })?;

        Some(DefinitionLocation::Dependency {
            file_id,
            range: dependency.range,
            selection_range: dependency.range,
        })
    }

    pub fn references_at(
        &self,
        file_id: FileId,
        offset: TextOffset,
        include_declaration: bool,
    ) -> Vec<ReferenceLocation> {
        let Some(name) = self.reference_name_at(file_id, offset) else {
            return Vec::new();
        };

        let mut locations = Vec::new();
        if include_declaration {
            locations.extend(
                self.dependencies(DependencyQuery::in_file(file_id))
                    .into_iter()
                    .filter(|dependency| dependency_matches_name(dependency, &name))
                    .map(|dependency| ReferenceLocation {
                        file_id,
                        range: dependency.range,
                    }),
            );
        }

        locations.extend(
            self.references(ReferenceQuery::in_file(file_id))
                .into_iter()
                .filter(|reference| reference.name == name)
                .map(|reference| ReferenceLocation {
                    file_id,
                    range: reference.range,
                }),
        );

        locations
    }

    fn reference_name_at(&self, file_id: FileId, offset: TextOffset) -> Option<String> {
        if let Some(reference) = self
            .references(ReferenceQuery::at(file_id, offset))
            .into_iter()
            .next()
        {
            return Some(reference.name.clone());
        }

        self.dependencies(DependencyQuery::in_file(file_id))
            .into_iter()
            .find(|dependency| dependency.range.contains(offset))
            .map(dependency_reference_name)
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

fn dependency_reference_name(dependency: &DependencyFact) -> String {
    dependency
        .alias
        .clone()
        .or_else(|| dependency.target.rsplit('\\').next().map(str::to_string))
        .unwrap_or_else(|| dependency.target.clone())
}

fn dependency_matches_name(dependency: &DependencyFact, name: &str) -> bool {
    dependency.alias.as_deref() == Some(name)
        || dependency
            .target
            .rsplit('\\')
            .next()
            .is_some_and(|target_name| target_name == name)
}

#[cfg(test)]
mod tests {
    use crate::{
        DefinitionLocation, DependencyFact, DependencyKind, DependencyQuery, EchoFileMode,
        EchoIndex, FileId, FqName, IndexFacts, IndexedFile, ReferenceFact, ReferenceKind,
        SymbolFact, SymbolKind, SymbolName, TextOffset, TextRange,
    };
    use smol_str::SmolStr;

    fn file(file_id: FileId, uri: &str) -> IndexedFile {
        IndexedFile {
            file_id,
            uri: uri.to_owned(),
            path: None,
            version: None,
            mode: EchoFileMode::Echo,
            content_hash: None,
        }
    }

    fn symbol(name: &str, kind: SymbolKind) -> SymbolFact {
        SymbolFact {
            name: SymbolName::new(name),
            fq_name: None,
            kind,
            range: TextRange::new(0, 10),
            selection_range: TextRange::new(0, 10),
            visibility: None,
            signature: None,
        }
    }

    fn fq_symbol(namespace: &[&str], name: &str, kind: SymbolKind) -> SymbolFact {
        SymbolFact {
            fq_name: Some(FqName::new(
                namespace.iter().map(|part| SmolStr::new(*part)).collect(),
                name,
            )),
            ..symbol(name, kind)
        }
    }

    #[test]
    fn returns_document_symbols_for_one_file() {
        let mut index = EchoIndex::new();
        let file_id = index.alloc_file_id();
        index.insert_file(file(file_id, "file:///one.echo"));
        index.update_file(
            file_id,
            IndexFacts::declarations(
                file_id,
                EchoFileMode::Echo,
                vec![
                    symbol("foo", SymbolKind::Function),
                    symbol("User", SymbolKind::Class),
                ],
            ),
        );

        let names = index
            .document_symbols(file_id)
            .into_iter()
            .map(|symbol| symbol.name.text.as_str())
            .collect::<Vec<_>>();

        assert_eq!(names, vec!["foo", "User"]);
    }

    #[test]
    fn returns_workspace_symbols_across_files() {
        let mut index = EchoIndex::new();
        let first = index.alloc_file_id();
        let second = index.alloc_file_id();
        index.insert_file(file(first, "file:///one.echo"));
        index.insert_file(file(second, "file:///two.echo"));
        index.update_file(
            first,
            IndexFacts::declarations(
                first,
                EchoFileMode::Echo,
                vec![symbol("foo", SymbolKind::Function)],
            ),
        );
        index.update_file(
            second,
            IndexFacts::declarations(
                second,
                EchoFileMode::Echo,
                vec![symbol("User", SymbolKind::Class)],
            ),
        );

        let symbols = index.workspace_symbols("User", 10);

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name.text.as_str(), "User");
    }

    #[test]
    fn update_replaces_only_symbols_from_that_file() {
        let mut index = EchoIndex::new();
        let first = index.alloc_file_id();
        let second = index.alloc_file_id();
        index.insert_file(file(first, "file:///one.echo"));
        index.insert_file(file(second, "file:///two.echo"));
        index.update_file(
            first,
            IndexFacts::declarations(
                first,
                EchoFileMode::Echo,
                vec![symbol("Old", SymbolKind::Class)],
            ),
        );
        index.update_file(
            second,
            IndexFacts::declarations(
                second,
                EchoFileMode::Echo,
                vec![symbol("Other", SymbolKind::Class)],
            ),
        );

        index.update_file(
            first,
            IndexFacts::declarations(
                first,
                EchoFileMode::Echo,
                vec![symbol("New", SymbolKind::Class)],
            ),
        );

        assert!(index.workspace_symbols("Old", 10).is_empty());
        assert_eq!(index.workspace_symbols("New", 10).len(), 1);
        assert_eq!(index.workspace_symbols("Other", 10).len(), 1);
    }

    #[test]
    fn remove_file_removes_its_symbols() {
        let mut index = EchoIndex::new();
        let file_id = index.alloc_file_id();
        index.insert_file(file(file_id, "file:///one.echo"));
        index.update_file(
            file_id,
            IndexFacts::declarations(
                file_id,
                EchoFileMode::Echo,
                vec![symbol("User", SymbolKind::Class)],
            ),
        );

        index.remove_file(file_id);

        assert!(index.document_symbols(file_id).is_empty());
        assert!(index.workspace_symbols("User", 10).is_empty());
    }

    #[test]
    fn fully_qualified_name_lookup_preserves_ambiguous_candidates() {
        let mut index = EchoIndex::new();
        let first = index.alloc_file_id();
        let second = index.alloc_file_id();
        let fq_name = FqName::new(vec![SmolStr::new("App")], "User");
        index.insert_file(file(first, "file:///one.php"));
        index.insert_file(file(second, "file:///two.php"));
        index.update_file(
            first,
            IndexFacts::declarations(
                first,
                EchoFileMode::PhpCompat,
                vec![fq_symbol(&["App"], "User", SymbolKind::Class)],
            ),
        );
        index.update_file(
            second,
            IndexFacts::declarations(
                second,
                EchoFileMode::PhpCompat,
                vec![fq_symbol(&["App"], "User", SymbolKind::Class)],
            ),
        );

        let candidates = index.symbols_by_fq_name(&fq_name);

        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].name.text.as_str(), "User");
        assert_eq!(candidates[1].name.text.as_str(), "User");
    }

    #[test]
    fn stores_dependency_facts_for_later_resolution() {
        let mut index = EchoIndex::new();
        let file_id = index.alloc_file_id();
        index.insert_file(file(file_id, "file:///one.echo"));
        index.update_file(
            file_id,
            IndexFacts {
                file_id,
                mode: EchoFileMode::PhpCompat,
                declarations: Vec::new(),
                dependencies: vec![DependencyFact {
                    kind: DependencyKind::PhpUse,
                    target: "Psr\\Log\\LoggerInterface".to_owned(),
                    alias: Some("LoggerInterface".to_owned()),
                    range: TextRange::new(5, 33),
                }],
                references: Vec::new(),
            },
        );

        let dependencies = index.dependencies(DependencyQuery::in_file(file_id));

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].kind, DependencyKind::PhpUse);
        assert_eq!(dependencies[0].target, "Psr\\Log\\LoggerInterface");
    }

    #[test]
    fn resolves_class_reference_to_php_use_dependency() {
        let mut index = EchoIndex::new();
        let file_id = index.alloc_file_id();
        index.insert_file(file(file_id, "file:///one.php"));
        index.update_file(
            file_id,
            IndexFacts {
                file_id,
                mode: EchoFileMode::PhpCompat,
                declarations: Vec::new(),
                dependencies: vec![DependencyFact {
                    kind: DependencyKind::PhpUse,
                    target: "Illuminate\\Http\\Request".to_string(),
                    alias: None,
                    range: TextRange::new(6, 36),
                }],
                references: vec![ReferenceFact {
                    kind: ReferenceKind::ClassLike,
                    name: "Request".to_string(),
                    range: TextRange::new(100, 107),
                }],
            },
        );

        let definition = index
            .definition_at(file_id, TextOffset(102))
            .expect("definition");

        assert_eq!(
            definition,
            DefinitionLocation::Dependency {
                file_id,
                range: TextRange::new(6, 36),
                selection_range: TextRange::new(6, 36),
            }
        );
    }

    #[test]
    fn returns_references_for_imported_class_reference() {
        let mut index = EchoIndex::new();
        let file_id = index.alloc_file_id();
        index.insert_file(file(file_id, "file:///one.php"));
        index.update_file(
            file_id,
            IndexFacts {
                file_id,
                mode: EchoFileMode::PhpCompat,
                declarations: Vec::new(),
                dependencies: vec![DependencyFact {
                    kind: DependencyKind::PhpUse,
                    target: "Illuminate\\Http\\Request".to_string(),
                    alias: None,
                    range: TextRange::new(6, 36),
                }],
                references: vec![ReferenceFact {
                    kind: ReferenceKind::ClassLike,
                    name: "Request".to_string(),
                    range: TextRange::new(100, 107),
                }],
            },
        );

        let references = index.references_at(file_id, TextOffset(102), true);

        assert_eq!(
            references,
            vec![
                crate::ReferenceLocation {
                    file_id,
                    range: TextRange::new(6, 36),
                },
                crate::ReferenceLocation {
                    file_id,
                    range: TextRange::new(100, 107),
                },
            ]
        );
    }

    #[test]
    fn resolves_method_definition_by_fully_qualified_class_name() {
        let mut index = EchoIndex::new();
        let file_id = index.alloc_file_id();
        index.insert_file(file(file_id, "file:///project/vendor/Application.php"));
        index.update_file(
            file_id,
            IndexFacts::declarations(
                file_id,
                EchoFileMode::PhpCompat,
                vec![fq_symbol(
                    &["Illuminate", "Foundation"],
                    "Application::handleRequest",
                    SymbolKind::Method,
                )],
            ),
        );

        let location = index
            .method_definition("Illuminate\\Foundation\\Application", "handleRequest")
            .expect("method definition");

        assert_eq!(location.file_id, file_id);
    }
}
