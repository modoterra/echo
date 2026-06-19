use std::sync::Mutex;

use dashmap::DashMap;
use echo_diagnostics::Diagnostic as EchoDiagnostic;
use echo_index::{EchoFileMode, EchoIndex, FileId, IndexFacts, IndexedFile};
use echo_source::SourceMode;
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::{
    Diagnostic, DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    InitializeParams, InitializeResult, InitializedParams, MessageType, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, Uri,
};
use tower_lsp_server::{Client, LanguageServer, LspService, Server};

use crate::diagnostics::diagnostics_to_lsp;
use crate::document::Document;

#[derive(Debug)]
pub struct Backend {
    client: Client,
    documents: DashMap<Uri, Document>,
    index: Mutex<EchoIndex>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: DashMap::new(),
            index: Mutex::new(EchoIndex::new()),
        }
    }

    async fn publish_document_diagnostics(&self, document: &Document) {
        let source = document.text_string();
        let diagnostics = match parse_index_facts(&source, document.file_id, document.mode) {
            Ok(facts) => {
                let mut index = self.index.lock().expect("echo index mutex poisoned");
                index.update_file(document.file_id, facts);
                Vec::new()
            }
            Err(diagnostics) => {
                let mut index = self.index.lock().expect("echo index mutex poisoned");
                index.update_file(
                    document.file_id,
                    echo_index::IndexFacts {
                        file_id: document.file_id,
                        mode: document.mode,
                        declarations: Vec::new(),
                        dependencies: Vec::new(),
                    },
                );
                diagnostics_to_lsp(&document.text, &diagnostics)
            }
        };

        self.publish_diagnostics(document.uri.clone(), diagnostics)
            .await;
    }

    async fn publish_diagnostics(&self, uri: Uri, diagnostics: Vec<Diagnostic>) {
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    #[cfg(test)]
    pub fn index_snapshot(&self) -> EchoIndex {
        self.index
            .lock()
            .expect("echo index mutex poisoned")
            .clone()
    }
}

impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
            offset_encoding: None,
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Echo language server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let text_document = params.text_document;
        let mode = crate::document::mode_from_uri(&text_document.uri);
        let file_id = {
            let mut index = self.index.lock().expect("echo index mutex poisoned");
            let file_id = index.alloc_file_id();
            index.insert_file(IndexedFile {
                file_id,
                uri: text_document.uri.to_string(),
                path: text_document
                    .uri
                    .to_file_path()
                    .map(|path| path.into_owned()),
                version: Some(text_document.version),
                mode,
                content_hash: None,
            });
            file_id
        };

        let document = Document::new(
            text_document.uri.clone(),
            text_document.version,
            text_document.text,
            file_id,
        );
        self.documents
            .insert(text_document.uri.clone(), document.clone());
        self.publish_document_diagnostics(&document).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;
        let Some(change) = params.content_changes.into_iter().last() else {
            return;
        };

        let document = {
            let Some(mut document) = self.documents.get_mut(&uri) else {
                return;
            };

            document.replace_text(version, change.text);
            document.clone()
        };

        self.publish_document_diagnostics(&document).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some((_, document)) = self.documents.remove(&uri) {
            let mut index = self.index.lock().expect("echo index mutex poisoned");
            index.remove_file(document.file_id);
        }

        self.publish_diagnostics(uri, Vec::new()).await;
    }
}

pub async fn run_stdio() -> std::io::Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
    Ok(())
}

fn source_mode(mode: EchoFileMode) -> SourceMode {
    match mode {
        EchoFileMode::Echo => SourceMode::Strict,
        EchoFileMode::PhpCompat => SourceMode::Echo,
    }
}

fn parse_index_facts(
    source: &str,
    file_id: FileId,
    mode: EchoFileMode,
) -> std::result::Result<IndexFacts, Vec<EchoDiagnostic>> {
    let program = echo_parser::parse_with_mode(source, source_mode(mode))?;
    Ok(echo_semantics::index_facts(&program, file_id, mode))
}

#[cfg(test)]
mod tests {
    use echo_index::{DependencyKind, SymbolKind};

    use super::*;

    #[test]
    fn parsed_document_produces_index_facts() {
        let facts = parse_index_facts(
            r#"<?php
namespace App\Http;
use Psr\Log\LoggerInterface as Logger;
fn handler(): string {
    echo "ok";
}
class UserController {
    function show(): string;
}
"#,
            FileId(3),
            EchoFileMode::PhpCompat,
        )
        .expect("source parses");

        assert_eq!(
            facts
                .declarations
                .iter()
                .map(|symbol| (symbol.name.text.as_str(), symbol.kind))
                .collect::<Vec<_>>(),
            vec![
                ("App\\Http", SymbolKind::Namespace),
                ("handler", SymbolKind::Function),
                ("UserController", SymbolKind::Class),
                ("show", SymbolKind::Method),
            ]
        );
        assert_eq!(facts.dependencies.len(), 1);
        assert_eq!(facts.dependencies[0].kind, DependencyKind::PhpUse);
        assert_eq!(facts.dependencies[0].target, "Psr\\Log\\LoggerInterface");
        assert_eq!(facts.dependencies[0].alias.as_deref(), Some("Logger"));
    }
}
