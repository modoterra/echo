use std::collections::HashMap;

use echo_index::{
    DependencyFact, DependencyKind, ReferenceLocation, Symbol, SymbolKind, TextOffset,
};
use ropey::Rope;
use tower_lsp_server::ls_types::{PrepareRenameResponse, Range, TextEdit, Uri, WorkspaceEdit};

use crate::position::range_to_lsp_range;

pub fn prepare_rename_at(
    text: &Rope,
    offset: TextOffset,
    symbols: &[&Symbol],
    dependencies: &[&DependencyFact],
    references: &[ReferenceLocation],
) -> Option<PrepareRenameResponse> {
    let target = rename_target_at(text, offset, symbols, dependencies, references)?;
    Some(PrepareRenameResponse::RangeWithPlaceholder {
        range: target.range,
        placeholder: target.placeholder,
    })
}

pub fn rename_workspace_edit(
    text: &Rope,
    uri: &Uri,
    offset: TextOffset,
    new_name: &str,
    symbols: &[&Symbol],
    dependencies: &[&DependencyFact],
    references: &[ReferenceLocation],
) -> Option<WorkspaceEdit> {
    let target = rename_target_at(text, offset, symbols, dependencies, references)?;
    let edits = match target.kind {
        RenameKind::Variable { name } => variable_rename_edits(text, &name, new_name),
        RenameKind::ClassLike { name } => {
            class_like_rename_edits(text, &name, new_name, dependencies, references)
        }
    };
    if edits.is_empty() {
        return None;
    }

    let mut changes = HashMap::new();
    changes.insert(uri.clone(), edits);
    Some(WorkspaceEdit::new(changes))
}

fn rename_target_at(
    text: &Rope,
    offset: TextOffset,
    symbols: &[&Symbol],
    dependencies: &[&DependencyFact],
    references: &[ReferenceLocation],
) -> Option<RenameTarget> {
    if let Some(variable) = variable_at(text, offset) {
        let range = range_to_lsp_range(
            text,
            echo_index::TextRange::new(variable.start as u32, variable.end as u32),
        );
        return Some(RenameTarget {
            kind: RenameKind::Variable {
                name: variable.name.clone(),
            },
            range,
            placeholder: format!("${}", variable.name),
        });
    }

    if let Some(symbol) = symbols
        .iter()
        .copied()
        .find(|symbol| symbol.selection_range.contains(offset))
    {
        if symbol.kind == SymbolKind::LocalVariable {
            return Some(RenameTarget {
                kind: RenameKind::Variable {
                    name: symbol.name.text.to_string(),
                },
                range: range_to_lsp_range(text, symbol.selection_range),
                placeholder: format!("${}", symbol.name.text),
            });
        }
    }

    dependencies
        .iter()
        .copied()
        .find(|dependency| {
            dependency.kind == DependencyKind::PhpUse && dependency.range.contains(offset)
        })
        .map(|dependency| {
            let name = dependency_alias_or_short_name(dependency);
            RenameTarget {
                kind: RenameKind::ClassLike { name: name.clone() },
                range: range_to_lsp_range(text, dependency.range),
                placeholder: name,
            }
        })
        .or_else(|| {
            references
                .iter()
                .find(|reference| reference.range.contains(offset))
                .map(|reference| RenameTarget {
                    kind: RenameKind::ClassLike {
                        name: text
                            .slice(reference.range.start as usize..reference.range.end as usize)
                            .to_string(),
                    },
                    range: range_to_lsp_range(text, reference.range),
                    placeholder: text
                        .slice(reference.range.start as usize..reference.range.end as usize)
                        .to_string(),
                })
        })
}

fn variable_rename_edits(text: &Rope, old_name: &str, new_name: &str) -> Vec<TextEdit> {
    let new_name = if new_name.starts_with('$') {
        new_name.to_string()
    } else {
        format!("${new_name}")
    };
    variable_occurrences(&text.to_string(), old_name)
        .into_iter()
        .map(|range| TextEdit {
            range: range_to_lsp_range(text, range),
            new_text: new_name.clone(),
        })
        .collect()
}

fn class_like_rename_edits(
    text: &Rope,
    old_name: &str,
    new_name: &str,
    dependencies: &[&DependencyFact],
    references: &[ReferenceLocation],
) -> Vec<TextEdit> {
    let mut edits = Vec::new();
    for dependency in dependencies {
        if dependency.kind != DependencyKind::PhpUse
            || dependency_alias_or_short_name(dependency) != old_name
        {
            continue;
        }
        edits.push(TextEdit {
            range: range_to_lsp_range(text, dependency.range),
            new_text: rewrite_use_dependency(text, dependency, new_name),
        });
    }
    for reference in references {
        edits.push(TextEdit {
            range: range_to_lsp_range(text, reference.range),
            new_text: new_name.to_string(),
        });
    }
    edits
}

fn dependency_alias_or_short_name(dependency: &DependencyFact) -> String {
    dependency
        .alias
        .clone()
        .or_else(|| dependency.target.rsplit('\\').next().map(str::to_string))
        .unwrap_or_else(|| dependency.target.clone())
}

fn rewrite_use_dependency(text: &Rope, dependency: &DependencyFact, new_name: &str) -> String {
    let source = text.to_string();
    let statement = &source[dependency.range.start as usize..dependency.range.end as usize];
    if dependency.alias.is_some() {
        return replace_last_identifier(statement, new_name);
    }

    let Some((prefix, _)) = dependency.target.rsplit_once('\\') else {
        return format!("use {new_name};");
    };
    format!("use {prefix}\\{new_name};")
}

fn replace_last_identifier(source: &str, new_name: &str) -> String {
    let bytes = source.as_bytes();
    let mut end = bytes.len();
    while end > 0 && !is_identifier_byte(bytes[end - 1]) {
        end -= 1;
    }
    let mut start = end;
    while start > 0 && is_identifier_byte(bytes[start - 1]) {
        start -= 1;
    }
    format!("{}{}{}", &source[..start], new_name, &source[end..])
}

fn variable_occurrences(source: &str, name: &str) -> Vec<echo_index::TextRange> {
    let needle = format!("${name}");
    let mut ranges = Vec::new();
    let mut search_start = 0;
    while let Some(relative) = source[search_start..].find(&needle) {
        let start = search_start + relative;
        let end = start + needle.len();
        let before_ok = start == 0 || !is_identifier_byte(source.as_bytes()[start - 1]);
        let after_ok = source
            .as_bytes()
            .get(end)
            .is_none_or(|byte| !is_identifier_byte(*byte));
        if before_ok && after_ok {
            ranges.push(echo_index::TextRange::new(start as u32, end as u32));
        }
        search_start = end;
    }
    ranges
}

fn variable_at(text: &Rope, offset: TextOffset) -> Option<VariableAt> {
    let source = text.to_string();
    let offset = offset.0 as usize;
    if offset > source.len() {
        return None;
    }
    let bytes = source.as_bytes();
    let mut start = offset.min(bytes.len());
    while start > 0 && is_identifier_byte(bytes[start - 1]) {
        start -= 1;
    }
    if start > 0 && bytes[start - 1] == b'$' {
        start -= 1;
    }
    if bytes.get(start) != Some(&b'$') {
        return None;
    }
    let mut end = start + 1;
    while end < bytes.len() && is_identifier_byte(bytes[end]) {
        end += 1;
    }
    if end == start + 1 || offset < start || offset > end {
        return None;
    }
    Some(VariableAt {
        name: source[start + 1..end].to_string(),
        start,
        end,
    })
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

#[derive(Debug, Clone)]
struct VariableAt {
    name: String,
    start: usize,
    end: usize,
}

#[derive(Debug, Clone)]
struct RenameTarget {
    kind: RenameKind,
    range: Range,
    placeholder: String,
}

#[derive(Debug, Clone)]
enum RenameKind {
    Variable { name: String },
    ClassLike { name: String },
}

#[cfg(test)]
mod tests {
    use echo_index::{
        DependencyFact, DependencyKind, FileId, ReferenceLocation, Signature, Symbol, SymbolId,
        SymbolName, TextRange,
    };
    use tower_lsp_server::ls_types::Position;

    use super::*;

    #[test]
    fn renames_php_variable_occurrences() {
        let text = Rope::from_str("/** @var Application $app */\n$app->handleRequest($app);\n");
        let edit = rename_workspace_edit(
            &text,
            &"file:///index.php".parse::<Uri>().unwrap(),
            TextOffset(31),
            "$kernel",
            &[],
            &[],
            &[],
        )
        .expect("edit");
        let edits = edit
            .changes
            .unwrap()
            .remove(&"file:///index.php".parse::<Uri>().unwrap())
            .unwrap();

        assert_eq!(edits.len(), 3);
        assert_eq!(edits[0].new_text, "$kernel");
    }

    #[test]
    fn prepares_phpdoc_local_variable_rename() {
        let text = Rope::from_str("/** @var Application $app */\n");
        let symbol = Symbol {
            id: SymbolId(1),
            file_id: FileId(1),
            name: SymbolName::new("app"),
            fq_name: None,
            kind: SymbolKind::LocalVariable,
            range: TextRange::new(8, 25),
            selection_range: TextRange::new(21, 25),
            visibility: None,
            container: None,
            signature: Some(Signature {
                text: "Application".to_string(),
            }),
        };

        let Some(PrepareRenameResponse::RangeWithPlaceholder { range, placeholder }) =
            prepare_rename_at(&text, TextOffset(22), &[&symbol], &[], &[])
        else {
            panic!("expected prepare rename response");
        };

        assert_eq!(placeholder, "$app");
        assert_eq!(range.start, Position::new(0, 21));
    }

    #[test]
    fn renames_imported_class_and_static_reference() {
        let text = Rope::from_str(
            "<?php\nuse Illuminate\\Foundation\\Application;\nuse Illuminate\\Http\\Request;\nRequest::capture();\n",
        );
        let application_dependency = DependencyFact {
            kind: DependencyKind::PhpUse,
            target: "Illuminate\\Foundation\\Application".to_string(),
            alias: None,
            range: TextRange::new(6, 43),
        };
        let request_dependency = DependencyFact {
            kind: DependencyKind::PhpUse,
            target: "Illuminate\\Http\\Request".to_string(),
            alias: None,
            range: TextRange::new(44, 74),
        };
        let reference = ReferenceLocation {
            file_id: FileId(1),
            range: TextRange::new(74, 81),
        };

        let edit = rename_workspace_edit(
            &text,
            &"file:///index.php".parse::<Uri>().unwrap(),
            TextOffset(76),
            "ServerRequest",
            &[],
            &[&application_dependency, &request_dependency],
            &[reference],
        )
        .expect("edit");
        let edits = edit
            .changes
            .unwrap()
            .remove(&"file:///index.php".parse::<Uri>().unwrap())
            .unwrap();

        assert_eq!(edits.len(), 2);
        assert_eq!(edits[0].new_text, "use Illuminate\\Http\\ServerRequest;");
        assert_eq!(edits[1].new_text, "ServerRequest");
    }
}
