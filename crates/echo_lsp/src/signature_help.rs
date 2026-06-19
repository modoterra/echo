use echo_index::{DependencyFact, DependencyKind, Symbol, SymbolKind, TextOffset};
use ropey::Rope;
use tower_lsp_server::ls_types::{
    ParameterInformation, ParameterLabel, SignatureHelp, SignatureHelpOptions,
    SignatureInformation, WorkDoneProgressOptions,
};

pub fn signature_help_options() -> SignatureHelpOptions {
    SignatureHelpOptions {
        trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
        retrigger_characters: Some(vec![",".to_string()]),
        work_done_progress_options: WorkDoneProgressOptions {
            work_done_progress: None,
        },
    }
}

pub fn signature_help_at(
    text: &Rope,
    offset: TextOffset,
    symbols: &[&Symbol],
    dependencies: &[&DependencyFact],
) -> Option<SignatureHelp> {
    let source = text.to_string();
    let call = call_context_at(&source, offset.0 as usize)?;
    let signature = signature_for_call(&call, symbols, dependencies)?;
    Some(SignatureHelp {
        signatures: vec![signature],
        active_signature: Some(0),
        active_parameter: Some(call.active_parameter),
    })
}

fn signature_for_call(
    call: &CallContext,
    symbols: &[&Symbol],
    dependencies: &[&DependencyFact],
) -> Option<SignatureInformation> {
    if call.receiver.is_none() && call.static_class.is_none() {
        if let Some(function) = echo_reflection::php_builtin(&call.name) {
            let params = function
                .params
                .iter()
                .map(|param| param.signature())
                .collect::<Vec<_>>();
            return Some(signature_information(
                format!("{}({})", function.name, params.join(", ")),
                params,
            ));
        }
    }

    if call.static_class.as_deref() == Some("Request")
        && imported_class_exists(dependencies, "Request")
        && call.name == "capture"
    {
        return Some(signature_information(
            "Request::capture(): Request".to_string(),
            Vec::new(),
        ));
    }

    if call.receiver.as_deref() == Some("$app")
        && local_variable_type(symbols, "app").is_some_and(|ty| ty.ends_with("Application"))
        && call.name == "handleRequest"
    {
        return Some(signature_information(
            "Application::handleRequest(Request $request): void".to_string(),
            vec!["Request $request".to_string()],
        ));
    }

    None
}

fn signature_information(label: String, params: Vec<String>) -> SignatureInformation {
    SignatureInformation {
        label,
        documentation: None,
        parameters: Some(
            params
                .into_iter()
                .map(|param| ParameterInformation {
                    label: ParameterLabel::Simple(param),
                    documentation: None,
                })
                .collect(),
        ),
        active_parameter: None,
    }
}

fn imported_class_exists(dependencies: &[&DependencyFact], name: &str) -> bool {
    dependencies.iter().any(|dependency| {
        dependency.kind == DependencyKind::PhpUse
            && (dependency.alias.as_deref() == Some(name)
                || dependency
                    .target
                    .rsplit('\\')
                    .next()
                    .is_some_and(|target_name| target_name == name))
    })
}

fn local_variable_type(symbols: &[&Symbol], name: &str) -> Option<String> {
    symbols
        .iter()
        .find(|symbol| symbol.kind == SymbolKind::LocalVariable && symbol.name.text == name)
        .and_then(|symbol| {
            symbol
                .signature
                .as_ref()
                .map(|signature| signature.text.clone())
        })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CallContext {
    receiver: Option<String>,
    static_class: Option<String>,
    name: String,
    active_parameter: u32,
}

fn call_context_at(source: &str, offset: usize) -> Option<CallContext> {
    let offset = offset.min(source.len());
    let open_paren = find_open_call_paren(source, offset)?;
    let active_parameter = active_parameter_index(&source[open_paren + 1..offset]);
    let callee_end = open_paren;
    let (name_start, name) = identifier_before(source, callee_end)?;

    if name_start >= 2 && &source[name_start - 2..name_start] == "::" {
        let (_, class_name) = identifier_before(source, name_start - 2)?;
        return Some(CallContext {
            receiver: None,
            static_class: Some(class_name),
            name,
            active_parameter,
        });
    }

    if name_start >= 2 && &source[name_start - 2..name_start] == "->" {
        let (receiver_start, receiver_name) = identifier_before(source, name_start - 2)?;
        let receiver = if receiver_start > 0 && source.as_bytes()[receiver_start - 1] == b'$' {
            format!("${receiver_name}")
        } else {
            receiver_name
        };
        return Some(CallContext {
            receiver: Some(receiver),
            static_class: None,
            name,
            active_parameter,
        });
    }

    Some(CallContext {
        receiver: None,
        static_class: None,
        name,
        active_parameter,
    })
}

fn find_open_call_paren(source: &str, offset: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut depth = 0usize;
    let mut index = offset;
    while index > 0 {
        index -= 1;
        match bytes[index] {
            b')' => depth += 1,
            b'(' if depth == 0 => return Some(index),
            b'(' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    None
}

fn active_parameter_index(argument_source: &str) -> u32 {
    let mut depth = 0usize;
    let mut active = 0u32;
    for byte in argument_source.bytes() {
        match byte {
            b'(' => depth += 1,
            b')' => depth = depth.saturating_sub(1),
            b',' if depth == 0 => active += 1,
            _ => {}
        }
    }
    active
}

fn identifier_before(source: &str, end: usize) -> Option<(usize, String)> {
    let bytes = source.as_bytes();
    let mut cursor = end.min(bytes.len());
    while cursor > 0 && bytes[cursor - 1].is_ascii_whitespace() {
        cursor -= 1;
    }
    let ident_end = cursor;
    while cursor > 0 && is_identifier_byte(bytes[cursor - 1]) {
        cursor -= 1;
    }
    if cursor == ident_end {
        return None;
    }
    Some((cursor, source[cursor..ident_end].to_string()))
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

#[cfg(test)]
mod tests {
    use echo_index::{DependencyFact, FileId, Signature, Symbol, SymbolId, SymbolName, TextRange};

    use super::*;

    #[test]
    fn returns_php_builtin_signature_help() {
        let help = signature_help_at(
            &Rope::from_str("<?php\nfile_exists($path);\n"),
            TextOffset(20),
            &[],
            &[],
        )
        .expect("signature help");

        assert_eq!(help.signatures[0].label, "file_exists(string $filename)");
        assert_eq!(help.active_parameter, Some(0));
    }

    #[test]
    fn returns_laravel_static_and_method_signature_help() {
        let dependency = DependencyFact {
            kind: DependencyKind::PhpUse,
            target: "Illuminate\\Http\\Request".to_string(),
            alias: None,
            range: TextRange::new(0, 30),
            target_range: TextRange::new(0, 30),
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

        let static_help = signature_help_at(
            &Rope::from_str("<?php\nRequest::capture();\n"),
            TextOffset(23),
            &[],
            &[&dependency],
        )
        .expect("static signature help");
        assert_eq!(
            static_help.signatures[0].label,
            "Request::capture(): Request"
        );

        let method_help = signature_help_at(
            &Rope::from_str("<?php\n$app->handleRequest(Request::capture());\n"),
            TextOffset(30),
            &[&app],
            &[&dependency],
        )
        .expect("method signature help");
        assert_eq!(
            method_help.signatures[0].label,
            "Application::handleRequest(Request $request): void"
        );
    }

    #[test]
    fn counts_active_parameter() {
        let help = signature_help_at(
            &Rope::from_str("<?php\ndefine('A', microtime(true));\n"),
            TextOffset(18),
            &[],
            &[],
        )
        .expect("signature help");

        assert_eq!(help.active_parameter, Some(1));
    }
}
