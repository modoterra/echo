use echo_index::{
    DefinitionLocation, DependencyFact, DependencyQuery, EchoIndex, FileId, ReferenceKind,
    ReferenceQuery, Symbol, TextOffset, TextRange,
};
use ropey::Rope;
use tower_lsp_server::ls_types::{Location, LocationLink, Uri};

use crate::definition_composer::composer_class_file;
use crate::definition_method::{local_variable_type, method_call_at, resolve_imported_type};
use crate::definition_target::{
    dependency_target_location, index_php_file, reference_target_location,
};
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

    dependency_target_location(index, &dependency)
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

    reference_target_location(index, file_id, reference)
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

#[cfg(test)]
#[path = "definition/tests.rs"]
mod tests;
