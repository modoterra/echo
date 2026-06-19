use std::collections::BTreeMap;

use echo_index::{DependencyFact, DependencyKind, Symbol, SymbolKind};
use tower_lsp_server::ls_types::{
    CompletionItem, CompletionItemKind, CompletionItemLabelDetails, CompletionOptions,
    WorkDoneProgressOptions,
};

pub fn completion_options() -> CompletionOptions {
    CompletionOptions {
        resolve_provider: Some(false),
        trigger_characters: Some(vec!["$".to_string(), ">".to_string(), ":".to_string()]),
        all_commit_characters: None,
        work_done_progress_options: WorkDoneProgressOptions {
            work_done_progress: None,
        },
        completion_item: None,
    }
}

pub fn completion_items(
    symbols: &[&Symbol],
    dependencies: &[&DependencyFact],
) -> Vec<CompletionItem> {
    let mut items = BTreeMap::new();

    for function in echo_reflection::php_builtins() {
        items.insert(
            format!("f:{}", function.name),
            CompletionItem {
                label: function.name.clone(),
                label_details: Some(CompletionItemLabelDetails {
                    detail: Some(format!("({})", function.params_signature())),
                    description: function.return_type.clone(),
                }),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("PHP builtin".to_string()),
                ..Default::default()
            },
        );
    }

    for dependency in dependencies {
        if dependency.kind != DependencyKind::PhpUse {
            continue;
        }
        let label = dependency
            .alias
            .clone()
            .or_else(|| dependency.target.rsplit('\\').next().map(str::to_string));
        let Some(label) = label else {
            continue;
        };
        items.insert(
            format!("c:{label}"),
            CompletionItem {
                label,
                kind: Some(CompletionItemKind::CLASS),
                detail: Some(dependency.target.clone()),
                ..Default::default()
            },
        );
    }

    for symbol in symbols {
        if symbol.kind != SymbolKind::LocalVariable {
            continue;
        }
        let label = format!("${}", symbol.name.text);
        items.insert(
            format!("v:{label}"),
            CompletionItem {
                label,
                kind: Some(CompletionItemKind::VARIABLE),
                detail: symbol
                    .signature
                    .as_ref()
                    .map(|signature| signature.text.clone()),
                ..Default::default()
            },
        );

        if symbol
            .signature
            .as_ref()
            .is_some_and(|signature| signature.text.ends_with("Application"))
        {
            items.insert(
                "m:handleRequest".to_string(),
                CompletionItem {
                    label: "handleRequest".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Application method".to_string()),
                    ..Default::default()
                },
            );
        }
    }

    items.into_values().collect()
}

#[cfg(test)]
mod tests {
    use echo_index::{DependencyFact, FileId, Signature, Symbol, SymbolId, SymbolName, TextRange};

    use super::*;

    #[test]
    fn includes_laravel_entrypoint_completion_items() {
        let dependency = DependencyFact {
            kind: DependencyKind::PhpUse,
            target: "Illuminate\\Http\\Request".to_string(),
            alias: None,
            range: TextRange::new(0, 30),
        };
        let app = Symbol {
            id: SymbolId(1),
            file_id: FileId(1),
            name: SymbolName::new("app"),
            fq_name: None,
            kind: SymbolKind::LocalVariable,
            range: TextRange::new(0, 25),
            selection_range: TextRange::new(20, 24),
            visibility: None,
            container: None,
            signature: Some(Signature {
                text: "Application".to_string(),
            }),
        };

        let items = completion_items(&[&app], &[&dependency]);
        let labels = items
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>();

        assert!(labels.contains(&"Request"));
        assert!(labels.contains(&"$app"));
        assert!(labels.contains(&"handleRequest"));
        assert!(labels.contains(&"file_exists"));
        assert!(labels.contains(&"microtime"));
        assert!(labels.contains(&"define"));
    }
}
