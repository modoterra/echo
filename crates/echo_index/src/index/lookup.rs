use crate::{
    DefinitionLocation, DependencyFact, DependencyQuery, EchoIndex, FileId, ReferenceFact,
    ReferenceLocation, ReferenceQuery, SymbolLocation, TextOffset,
};

impl EchoIndex {
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
            if dependency.target_range.contains(offset) {
                return Some(DefinitionLocation::Dependency {
                    file_id,
                    range: dependency.range,
                    selection_range: dependency.target_range,
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
            selection_range: dependency.target_range,
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
