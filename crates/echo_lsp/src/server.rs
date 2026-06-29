use std::path::{Path, PathBuf};
use std::sync::Mutex;

use dashmap::DashMap;
use echo_index::{DependencyQuery, EchoIndex, IndexedFile, ReferenceQuery, TextOffset};
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::{
    CompletionParams, CompletionResponse, Diagnostic, DidChangeTextDocumentParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, DocumentLink, DocumentLinkOptions,
    DocumentLinkParams, DocumentSymbolParams, DocumentSymbolResponse, GotoDefinitionParams,
    GotoDefinitionResponse, Hover, HoverParams, HoverProviderCapability, InitializeParams,
    InitializeResult, InitializedParams, Location, MessageType, OneOf, PrepareRenameResponse,
    ReferenceParams, RenameOptions, RenameParams, SemanticTokensParams, SemanticTokensResult,
    SemanticTokensServerCapabilities, ServerCapabilities, SignatureHelp, SignatureHelpParams,
    TextDocumentPositionParams, TextDocumentSyncCapability, TextDocumentSyncKind, Uri,
    WorkDoneProgressOptions, WorkspaceEdit, WorkspaceSymbolParams, WorkspaceSymbolResponse,
};
use tower_lsp_server::{Client, LanguageServer, LspService, Server};

use crate::completion::{completion_items, completion_options};
use crate::definition::{
    definition_location_to_lsp, dependency_target_link_at, method_definition_at,
    receiver_method_definition_at, reference_target_link_at,
};
use crate::diagnostics::diagnostics_to_lsp;
use crate::document::Document;
use crate::document_links::document_links_for_paths;
use crate::hover::hover_at;
use crate::indexing::{index_required_files, parse_index_facts};
use crate::position::position_to_offset;
use crate::references::reference_locations_to_lsp;
use crate::rename::{prepare_rename_at, rename_workspace_edit};
use crate::semantic_tokens::{semantic_tokens_for_source, semantic_tokens_options};
use crate::signature_help::{signature_help_at, signature_help_options};
use crate::symbols::{document_symbols_to_lsp, workspace_symbols_to_lsp};

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
        let source_dir = document_source_dir(document);
        let diagnostics = match parse_index_facts(&source, document.file_id, source_dir.as_deref())
        {
            Ok(facts) => {
                let mut index = self.index.lock().expect("echo index mutex poisoned");
                index.update_file(document.file_id, facts);
                index_required_files(&mut index, document.file_id);
                Vec::new()
            }
            Err(diagnostics) => {
                let mut index = self.index.lock().expect("echo index mutex poisoned");
                index.update_file(
                    document.file_id,
                    echo_index::IndexFacts {
                        file_id: document.file_id,
                        declarations: Vec::new(),
                        dependencies: Vec::new(),
                        references: Vec::new(),
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
                completion_provider: Some(completion_options()),
                document_symbol_provider: Some(OneOf::Left(true)),
                definition_provider: Some(OneOf::Left(true)),
                document_link_provider: Some(DocumentLinkOptions {
                    resolve_provider: Some(false),
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: None,
                    },
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                references_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Right(RenameOptions {
                    prepare_provider: Some(true),
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: None,
                    },
                })),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        semantic_tokens_options(),
                    ),
                ),
                signature_help_provider: Some(signature_help_options()),
                workspace_symbol_provider: Some(OneOf::Left(true)),
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

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;
        let Some(document) = self.documents.get(&uri) else {
            return Ok(Some(DocumentSymbolResponse::Nested(Vec::new())));
        };

        let symbols = {
            let index = self.index.lock().expect("echo index mutex poisoned");
            document_symbols_to_lsp(&document.text, &index.document_symbols(document.file_id))
        };

        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }

    async fn document_link(&self, params: DocumentLinkParams) -> Result<Option<Vec<DocumentLink>>> {
        let uri = params.text_document.uri;
        let Some(document) = self.documents.get(&uri) else {
            return Ok(Some(Vec::new()));
        };

        let links = {
            let index = self.index.lock().expect("echo index mutex poisoned");
            document_links_for_paths(
                &document.text,
                &index.dependencies(DependencyQuery::in_file(document.file_id)),
                &index.references(ReferenceQuery::in_file(document.file_id)),
            )
        };

        Ok(Some(links))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let position_params = params.text_document_position_params;
        let uri = position_params.text_document.uri;
        let Some(document) = self.documents.get(&uri) else {
            return Ok(None);
        };
        let Some(offset) = position_to_offset(&document.text, position_params.position) else {
            return Ok(None);
        };
        let offset = TextOffset(offset as u32);

        let hover = {
            let index = self.index.lock().expect("echo index mutex poisoned");
            hover_at(
                &document.text,
                offset,
                &index.document_symbols(document.file_id),
                &index.dependencies(DependencyQuery::in_file(document.file_id)),
                &index.references(echo_index::ReferenceQuery::in_file(document.file_id)),
            )
        };

        Ok(hover)
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<WorkspaceSymbolResponse>> {
        let symbols = {
            let index = self.index.lock().expect("echo index mutex poisoned");
            workspace_symbols_to_lsp(&index, &params.query, 100)
        };

        Ok(Some(WorkspaceSymbolResponse::Flat(symbols)))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let position_params = params.text_document_position_params;
        let uri = position_params.text_document.uri;
        let Some(document) = self.documents.get(&uri) else {
            return Ok(None);
        };
        let Some(offset) = position_to_offset(&document.text, position_params.position) else {
            return Ok(None);
        };

        let location = {
            let mut index = self.index.lock().expect("echo index mutex poisoned");
            let text_offset = TextOffset(offset as u32);
            if let Some(link) =
                reference_target_link_at(&mut index, &document.text, document.file_id, text_offset)
            {
                return Ok(Some(GotoDefinitionResponse::Link(vec![link])));
            }
            if let Some(link) =
                dependency_target_link_at(&mut index, &document.text, document.file_id, text_offset)
            {
                return Ok(Some(GotoDefinitionResponse::Link(vec![link])));
            }
            let document_symbols = index
                .document_symbols(document.file_id)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>();
            let dependencies = index
                .dependencies(DependencyQuery::in_file(document.file_id))
                .into_iter()
                .cloned()
                .collect::<Vec<_>>();
            let definition = receiver_method_definition_at(
                &mut index,
                document.file_id,
                text_offset,
                &document_symbols,
                &dependencies,
            )
            .or_else(|| {
                method_definition_at(
                    &index,
                    &document.text,
                    text_offset,
                    &document_symbols,
                    &dependencies,
                )
            })
            .or_else(|| index.definition_at(document.file_id, text_offset));
            let Some(definition) = definition else {
                return Ok(None);
            };
            definition_location_to_lsp(&index, &document.text, document.file_id, &uri, definition)
        };

        Ok(location.map(GotoDefinitionResponse::Scalar))
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let position_params = params.text_document_position;
        let uri = position_params.text_document.uri;
        let Some(document) = self.documents.get(&uri) else {
            return Ok(None);
        };
        let Some(offset) = position_to_offset(&document.text, position_params.position) else {
            return Ok(None);
        };

        let locations = {
            let index = self.index.lock().expect("echo index mutex poisoned");
            let references = index.references_at(
                document.file_id,
                TextOffset(offset as u32),
                params.context.include_declaration,
            );
            reference_locations_to_lsp(&document.text, document.file_id, &uri, &references)
        };

        Ok(Some(locations))
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri;
        let Some(document) = self.documents.get(&uri) else {
            return Ok(None);
        };

        Ok(Some(SemanticTokensResult::Tokens(
            semantic_tokens_for_source(&document.text_string()),
        )))
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let Some(document) = self.documents.get(&uri) else {
            return Ok(None);
        };

        let items = {
            let index = self.index.lock().expect("echo index mutex poisoned");
            completion_items(
                &index.document_symbols(document.file_id),
                &index.dependencies(DependencyQuery::in_file(document.file_id)),
            )
        };

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        let position_params = params.text_document_position_params;
        let uri = position_params.text_document.uri;
        let Some(document) = self.documents.get(&uri) else {
            return Ok(None);
        };
        let Some(offset) = position_to_offset(&document.text, position_params.position) else {
            return Ok(None);
        };

        let help = {
            let index = self.index.lock().expect("echo index mutex poisoned");
            signature_help_at(
                &document.text,
                TextOffset(offset as u32),
                &index.document_symbols(document.file_id),
                &index.dependencies(DependencyQuery::in_file(document.file_id)),
            )
        };

        Ok(help)
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let position_params = params.text_document_position;
        let uri = position_params.text_document.uri;
        let Some(document) = self.documents.get(&uri) else {
            return Ok(None);
        };
        let Some(offset) = position_to_offset(&document.text, position_params.position) else {
            return Ok(None);
        };

        let edit = {
            let index = self.index.lock().expect("echo index mutex poisoned");
            let references = index.references_at(document.file_id, TextOffset(offset as u32), true);
            rename_workspace_edit(
                &document.text,
                &uri,
                TextOffset(offset as u32),
                &params.new_name,
                &index.document_symbols(document.file_id),
                &index.dependencies(DependencyQuery::in_file(document.file_id)),
                &references,
            )
        };

        Ok(edit)
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        let position_params = params;
        let uri = position_params.text_document.uri;
        let Some(document) = self.documents.get(&uri) else {
            return Ok(None);
        };
        let Some(offset) = position_to_offset(&document.text, position_params.position) else {
            return Ok(None);
        };

        let response = {
            let index = self.index.lock().expect("echo index mutex poisoned");
            let references = index.references_at(document.file_id, TextOffset(offset as u32), true);
            prepare_rename_at(
                &document.text,
                TextOffset(offset as u32),
                &index.document_symbols(document.file_id),
                &index.dependencies(DependencyQuery::in_file(document.file_id)),
                &references,
            )
        };

        Ok(response)
    }
}

pub async fn run_stdio() -> std::io::Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
    Ok(())
}

fn document_source_dir(document: &Document) -> Option<PathBuf> {
    document
        .uri
        .to_file_path()
        .and_then(|path| path.parent().map(Path::to_path_buf))
}

#[cfg(test)]
#[path = "server/tests.rs"]
mod tests;
