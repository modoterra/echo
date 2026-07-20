//! LSP over stdio (JSON-RPC 2.0, Content-Length framing).
//!
//! Feature handlers are presentation over the shared pipeline — see `features`.

use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::analysis::analyze_path;
use crate::document::{uri_to_path, DocumentStore};
use crate::features::{
    analysis_product, completion, definition, document_symbols, format_edits, hover, module_for_path,
    references, rename, semantic_tokens_with_ast, signature_help, workspace_symbols, Location,
    LspRange, RenameResult, SEMANTIC_TOKEN_MODIFIERS, SEMANTIC_TOKEN_TYPES,
};
use crate::position::Position;
use crate::LspSeverity;

/// Run the language server on stdin/stdout until `exit`.
pub fn run_stdio() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    let mut store = DocumentStore::new();
    let mut root: Option<PathBuf> = None;

    loop {
        let Some(msg) = read_message(&mut stdin)? else {
            break;
        };
        let method = msg.get("method").and_then(|m| m.as_str());
        let id = msg.get("id").cloned();

        match method {
            Some("initialize") => {
                if let Some(params) = msg.get("params") {
                    if let Some(uri) = params
                        .pointer("/rootUri")
                        .and_then(|u| u.as_str())
                        .and_then(uri_to_path)
                    {
                        root = Some(uri);
                    } else if let Some(path) = params
                        .pointer("/rootPath")
                        .and_then(|u| u.as_str())
                        .map(PathBuf::from)
                    {
                        root = Some(path);
                    }
                }
                write_result(&mut stdout, id, initialize_capabilities())?;
            }
            Some("initialized") | Some("workspace/didChangeConfiguration") => {}
            Some("shutdown") => {
                write_result(&mut stdout, id, Value::Null)?;
            }
            Some("exit") => break,
            Some("textDocument/didOpen") => {
                if let Some(params) = msg.get("params") {
                    handle_did_open(&mut store, params, &mut stdout)?;
                }
            }
            Some("textDocument/didChange") => {
                if let Some(params) = msg.get("params") {
                    handle_did_change(&mut store, params, &mut stdout)?;
                }
            }
            Some("textDocument/didClose") => {
                if let Some(params) = msg.get("params") {
                    handle_did_close(&mut store, params, &mut stdout)?;
                }
            }
            Some("textDocument/didSave") => {
                if let Some(params) = msg.get("params") {
                    handle_did_save(&mut store, params, &mut stdout)?;
                }
            }
            Some("textDocument/hover") => {
                let result = with_doc_pos(&store, &msg, |path, text, pos, product| {
                    hover(product, path, text, pos).map(|h| {
                        json!({
                            "contents": { "kind": "markdown", "value": h.contents },
                            "range": range_json(h.range),
                        })
                    })
                });
                write_result(&mut stdout, id, result.unwrap_or(Value::Null))?;
            }
            Some("textDocument/definition") => {
                let result = with_doc_pos(&store, &msg, |path, text, pos, product| {
                    definition(product, path, text, pos).map(location_json)
                });
                write_result(&mut stdout, id, result.unwrap_or(Value::Null))?;
            }
            Some("textDocument/references") => {
                let include_decl = msg
                    .pointer("/params/context/includeDeclaration")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                let result = with_doc_pos(&store, &msg, |path, text, pos, product| {
                    let locs = references(product, path, text, pos, include_decl);
                    Some(Value::Array(locs.into_iter().map(location_json).collect()))
                });
                write_result(
                    &mut stdout,
                    id,
                    result.unwrap_or_else(|| Value::Array(vec![])),
                )?;
            }
            Some("textDocument/completion") => {
                let result = with_doc_pos(&store, &msg, |path, text, pos, product| {
                    let items = completion(product, path, text, pos);
                    Some(Value::Array(
                        items
                            .into_iter()
                            .map(|c| {
                                let mut o = json!({
                                    "label": c.label,
                                    "kind": c.kind as i32,
                                });
                                if let Some(d) = c.detail {
                                    o["detail"] = json!(d);
                                }
                                if let Some(t) = c.insert_text {
                                    o["insertText"] = json!(t);
                                }
                                o
                            })
                            .collect(),
                    ))
                });
                write_result(
                    &mut stdout,
                    id,
                    result.unwrap_or_else(|| Value::Array(vec![])),
                )?;
            }
            Some("textDocument/signatureHelp") => {
                let result = with_doc_pos(&store, &msg, |path, text, pos, product| {
                    signature_help(product, path, text, pos).map(|h| {
                        json!({
                            "signatures": [{
                                "label": h.label,
                                "parameters": h.parameters.iter().map(|p| json!({"label": p})).collect::<Vec<_>>(),
                            }],
                            "activeSignature": 0,
                            "activeParameter": h.active_parameter,
                        })
                    })
                });
                write_result(&mut stdout, id, result.unwrap_or(Value::Null))?;
            }
            Some("textDocument/rename") => {
                let new_name = msg
                    .pointer("/params/newName")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                match with_doc_pos(&store, &msg, |path, text, pos, product| {
                    Some(rename(product, path, text, pos, new_name))
                }) {
                    Some(RenameResult::Ok(edit)) => {
                        let edits: Vec<Value> = edit
                            .edits
                            .into_iter()
                            .map(|e| {
                                json!({
                                    "range": range_json(e.range),
                                    "newText": e.new_text,
                                })
                            })
                            .collect();
                        write_result(
                            &mut stdout,
                            id,
                            json!({
                                "changes": {
                                    edit.uri: edits
                                }
                            }),
                        )?;
                    }
                    Some(RenameResult::Err(msg)) => {
                        write_error(&mut stdout, id, -32602, msg)?;
                    }
                    None => write_result(&mut stdout, id, Value::Null)?,
                }
            }
            Some("textDocument/documentSymbol") => {
                let result = with_doc(&store, &msg, |path, text, product| {
                    let Some(m) = module_for_path(product, path) else {
                        return Some(Value::Array(vec![]));
                    };
                    let Some(file) = m.file.as_ref() else {
                        return Some(Value::Array(vec![]));
                    };
                    let syms = document_symbols(path, text, file);
                    Some(Value::Array(
                        syms.into_iter()
                            .map(|s| {
                                json!({
                                    "name": s.name,
                                    "kind": s.kind as i32,
                                    "range": range_json(s.range),
                                    "selectionRange": range_json(s.selection_range),
                                    "detail": s.container,
                                })
                            })
                            .collect(),
                    ))
                });
                write_result(
                    &mut stdout,
                    id,
                    result.unwrap_or_else(|| Value::Array(vec![])),
                )?;
            }
            Some("workspace/symbol") => {
                let query = msg
                    .pointer("/params/query")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                // Use any open doc as entry, else workspace root placeholder.
                let result = if let Some(doc) = store.iter().next() {
                    if let Some(path) = doc.path.as_ref() {
                        let overlays = store.overlays();
                        let product = analysis_product(path, &overlays, true);
                        let syms = workspace_symbols(&product, &overlays, query);
                        Value::Array(
                            syms.into_iter()
                                .map(|s| {
                                    json!({
                                        "name": s.name,
                                        "kind": s.kind as i32,
                                        "location": {
                                            "uri": s.uri,
                                            "range": range_json(s.range),
                                        },
                                        "containerName": s.container,
                                    })
                                })
                                .collect(),
                        )
                    } else {
                        Value::Array(vec![])
                    }
                } else if let Some(r) = root.as_ref() {
                    // No open docs — empty
                    let _ = r;
                    Value::Array(vec![])
                } else {
                    Value::Array(vec![])
                };
                write_result(&mut stdout, id, result)?;
            }
            Some("textDocument/formatting") => {
                let result = with_doc_text(&store, &msg, |path, text| {
                    match format_edits(path, text) {
                        Ok(edits) => Some(Value::Array(
                            edits
                                .into_iter()
                                .map(|e| {
                                    json!({
                                        "range": range_json(e.range),
                                        "newText": e.new_text,
                                    })
                                })
                                .collect(),
                        )),
                        Err(_) => Some(Value::Array(vec![])),
                    }
                });
                write_result(
                    &mut stdout,
                    id,
                    result.unwrap_or_else(|| Value::Array(vec![])),
                )?;
            }
            Some("textDocument/semanticTokens/full") => {
                let result = with_doc(&store, &msg, |path, text, product| {
                    let file = module_for_path(product, path).and_then(|m| m.file.as_ref());
                    let data = semantic_tokens_with_ast(text, file);
                    Some(json!({ "data": data }))
                });
                write_result(
                    &mut stdout,
                    id,
                    result.unwrap_or_else(|| json!({ "data": [] })),
                )?;
            }
            Some(other) if id.is_some() => {
                write_error(
                    &mut stdout,
                    id,
                    -32601,
                    format!("method not found: {other}"),
                )?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn initialize_capabilities() -> Value {
    json!({
        "capabilities": {
            "textDocumentSync": {
                "openClose": true,
                "change": 1,
                "save": { "includeText": true }
            },
            "hoverProvider": true,
            "definitionProvider": true,
            "referencesProvider": true,
            "completionProvider": {
                "triggerCharacters": [".", "/", "$", "~", "#", "%"]
            },
            "signatureHelpProvider": {
                "triggerCharacters": ["(", ","]
            },
            "renameProvider": true,
            "documentSymbolProvider": true,
            "workspaceSymbolProvider": true,
            "documentFormattingProvider": true,
            "semanticTokensProvider": {
                "legend": {
                    "tokenTypes": SEMANTIC_TOKEN_TYPES,
                    "tokenModifiers": SEMANTIC_TOKEN_MODIFIERS,
                },
                "full": true,
            }
        },
        "serverInfo": {
            "name": "echo-lsp",
            "version": env!("CARGO_PKG_VERSION")
        }
    })
}

fn range_json(r: LspRange) -> Value {
    json!({
        "start": { "line": r.start.line, "character": r.start.character },
        "end": { "line": r.end.line, "character": r.end.character },
    })
}

fn location_json(loc: Location) -> Value {
    json!({
        "uri": loc.uri,
        "range": range_json(loc.range),
    })
}

fn pos_from_params(params: &Value) -> Option<Position> {
    let line = params.pointer("/position/line")?.as_u64()? as u32;
    let character = params.pointer("/position/character")?.as_u64()? as u32;
    Some(Position { line, character })
}

fn with_doc_pos<F, R>(store: &DocumentStore, msg: &Value, f: F) -> Option<R>
where
    F: FnOnce(&Path, &str, Position, &echo_pipeline::AnalysisProduct) -> Option<R>,
{
    let params = msg.get("params")?;
    let uri = params.pointer("/textDocument/uri")?.as_str()?;
    let pos = pos_from_params(params)?;
    let doc = store.get(uri)?;
    let path = doc.path.as_ref()?;
    let overlays = store.overlays();
    let product = analysis_product(path, &overlays, true);
    f(path, &doc.text, pos, &product)
}

fn with_doc<F, R>(store: &DocumentStore, msg: &Value, f: F) -> Option<R>
where
    F: FnOnce(&Path, &str, &echo_pipeline::AnalysisProduct) -> Option<R>,
{
    let params = msg.get("params")?;
    let uri = params.pointer("/textDocument/uri")?.as_str()?;
    let doc = store.get(uri)?;
    let path = doc.path.as_ref()?;
    let overlays = store.overlays();
    let product = analysis_product(path, &overlays, true);
    f(path, &doc.text, &product)
}

fn with_doc_text<F, R>(store: &DocumentStore, msg: &Value, f: F) -> Option<R>
where
    F: FnOnce(&Path, &str) -> Option<R>,
{
    let params = msg.get("params")?;
    let uri = params.pointer("/textDocument/uri")?.as_str()?;
    let doc = store.get(uri)?;
    let path = doc.path.as_ref()?;
    f(path, &doc.text)
}

fn handle_did_open(
    store: &mut DocumentStore,
    params: &Value,
    out: &mut impl Write,
) -> io::Result<()> {
    let doc = &params["textDocument"];
    let uri = doc["uri"].as_str().unwrap_or("").to_string();
    let version = doc["version"].as_i64().unwrap_or(0) as i32;
    let text = doc["text"].as_str().unwrap_or("").to_string();
    store.open(uri.clone(), version, text);
    publish_for_uri(store, &uri, out)
}

fn handle_did_change(
    store: &mut DocumentStore,
    params: &Value,
    out: &mut impl Write,
) -> io::Result<()> {
    let uri = params["textDocument"]["uri"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let version = params["textDocument"]["version"].as_i64().unwrap_or(0) as i32;
    let text = params["contentChanges"]
        .as_array()
        .and_then(|a| a.last())
        .and_then(|c| c.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();
    store.change(&uri, version, text);
    publish_for_uri(store, &uri, out)
}

fn handle_did_close(
    store: &mut DocumentStore,
    params: &Value,
    out: &mut impl Write,
) -> io::Result<()> {
    let uri = params["textDocument"]["uri"]
        .as_str()
        .unwrap_or("")
        .to_string();
    store.close(&uri);
    publish_diagnostics(out, &uri, Vec::new())?;
    Ok(())
}

fn handle_did_save(
    store: &mut DocumentStore,
    params: &Value,
    out: &mut impl Write,
) -> io::Result<()> {
    let uri = params["textDocument"]["uri"]
        .as_str()
        .unwrap_or("")
        .to_string();
    if let Some(text) = params.get("text").and_then(|t| t.as_str()) {
        let ver = store.get(&uri).map(|d| d.version).unwrap_or(0);
        store.change(&uri, ver, text.to_string());
    }
    publish_for_uri(store, &uri, out)
}

fn publish_for_uri(
    store: &DocumentStore,
    uri: &str,
    out: &mut impl Write,
) -> io::Result<()> {
    let Some(doc) = store.get(uri) else {
        return Ok(());
    };
    let Some(path) = doc.path.as_ref() else {
        return Ok(());
    };
    let entry = path.clone();
    let overlays = store.overlays();
    let diags = analyze_path(&entry, &overlays, true);
    let for_doc: Vec<_> = diags
        .into_iter()
        .filter(|d| {
            d.uri == *uri
                || uri_to_path(&d.uri).and_then(|p| p.canonicalize().ok())
                    == path.canonicalize().ok()
        })
        .collect();
    let list = if for_doc.is_empty() {
        analyze_path(&entry, &overlays, true)
            .into_iter()
            .filter(|d| {
                d.uri
                    .contains(path.file_name().and_then(|s| s.to_str()).unwrap_or(""))
            })
            .collect()
    } else {
        for_doc
    };
    publish_diagnostics(out, uri, list)
}

fn publish_diagnostics(
    out: &mut impl Write,
    uri: &str,
    diags: Vec<crate::LspDiagnostic>,
) -> io::Result<()> {
    let items: Vec<Value> = diags
        .iter()
        .map(|d| {
            let severity = match d.severity {
                LspSeverity::Error => 1,
                LspSeverity::Warning => 2,
                LspSeverity::Information => 3,
                LspSeverity::Hint => 4,
            };
            let mut obj = json!({
                "range": {
                    "start": { "line": d.start.line, "character": d.start.character },
                    "end": { "line": d.end.line, "character": d.end.character }
                },
                "severity": severity,
                "source": "echo",
                "message": d.message,
            });
            if let Some(code) = &d.code {
                obj["code"] = json!(code);
            }
            obj
        })
        .collect();

    let note = json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {
            "uri": uri,
            "diagnostics": items,
        }
    });
    write_message(out, &note)
}

fn read_message(reader: &mut impl BufRead) -> io::Result<Option<Value>> {
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            return Ok(None);
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some(rest) = line.strip_prefix("Content-Length:") {
            content_length = rest.trim().parse().ok();
        }
    }
    let len = content_length.ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "missing Content-Length")
    })?;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    let v: Value = serde_json::from_slice(&buf)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(Some(v))
}

fn write_message(out: &mut impl Write, value: &Value) -> io::Result<()> {
    let body = serde_json::to_vec(value)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    write!(out, "Content-Length: {}\r\n\r\n", body.len())?;
    out.write_all(&body)?;
    out.flush()?;
    Ok(())
}

fn write_result(out: &mut impl Write, id: Option<Value>, result: Value) -> io::Result<()> {
    let Some(id) = id else {
        return Ok(());
    };
    write_message(
        out,
        &json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result,
        }),
    )
}

fn write_error(
    out: &mut impl Write,
    id: Option<Value>,
    code: i32,
    message: String,
) -> io::Result<()> {
    let Some(id) = id else {
        return Ok(());
    };
    write_message(
        out,
        &json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": code, "message": message },
        }),
    )
}


