use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use dashmap::DashMap;
use echo_diagnostics::Diagnostic as EchoDiagnostic;
use echo_index::{
    DependencyQuery, EchoFileMode, EchoIndex, FileId, IndexFacts, IndexedFile, ReferenceQuery,
    TextOffset,
};
use echo_source::SourceMode;
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
        let diagnostics = match parse_index_facts(
            &source,
            document.file_id,
            document.mode,
            source_dir.as_deref(),
        ) {
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
                        mode: document.mode,
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
    source_dir: Option<&Path>,
) -> std::result::Result<IndexFacts, Vec<EchoDiagnostic>> {
    let mut program = echo_parser::parse_with_mode(source, source_mode(mode))?;
    program.source_dir = source_dir.map(|path| path.to_string_lossy().to_string());
    Ok(echo_semantics::index_facts_from_source(
        source, &program, file_id, mode,
    ))
}

fn document_source_dir(document: &Document) -> Option<PathBuf> {
    document
        .uri
        .to_file_path()
        .and_then(|path| path.parent().map(Path::to_path_buf))
}

fn index_required_files(index: &mut EchoIndex, root_file_id: FileId) {
    let mut visited = HashSet::new();
    index_required_files_inner(index, root_file_id, &mut visited);
}

fn index_required_files_inner(
    index: &mut EchoIndex,
    file_id: FileId,
    visited: &mut HashSet<PathBuf>,
) {
    let dependencies = index
        .dependencies(DependencyQuery::in_file(file_id))
        .into_iter()
        .filter(|dependency| {
            matches!(
                dependency.kind,
                echo_index::DependencyKind::Require
                    | echo_index::DependencyKind::RequireOnce
                    | echo_index::DependencyKind::Include
                    | echo_index::DependencyKind::IncludeOnce
                    | echo_index::DependencyKind::ComposerAutoload
            )
        })
        .map(|dependency| dependency.target.clone())
        .collect::<Vec<_>>();

    for target in dependencies {
        let path = PathBuf::from(target);
        let Ok(path) = std::fs::canonicalize(path) else {
            continue;
        };
        if !visited.insert(path.clone())
            || path.extension().and_then(|ext| ext.to_str()) != Some("php")
        {
            continue;
        }

        let included_file_id = match index.file_by_path(&path).map(|file| file.file_id) {
            Some(file_id) => file_id,
            None => {
                let file_id = index.alloc_file_id();
                let uri = Uri::from_file_path(&path)
                    .map(|uri| uri.to_string())
                    .unwrap_or_else(|| format!("file://{}", path.display()));
                index.insert_file(IndexedFile {
                    file_id,
                    uri,
                    path: Some(path.clone()),
                    version: None,
                    mode: EchoFileMode::PhpCompat,
                    content_hash: None,
                });
                file_id
            }
        };

        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        let source_dir = path.parent();
        if let Ok(facts) = parse_index_facts(
            &source,
            included_file_id,
            EchoFileMode::PhpCompat,
            source_dir,
        ) {
            index.update_file(included_file_id, facts);
            index_required_files_inner(index, included_file_id, visited);
        }
    }
}

#[cfg(test)]
mod tests {
    use echo_index::{DependencyKind, ReferenceKind, SymbolKind, TextRange};
    use ropey::Rope;

    use crate::position::range_to_lsp_range;

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
            None,
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

    #[test]
    fn laravel_entrypoint_produces_import_and_static_reference_facts() {
        let source = r#"<?php

use Illuminate\Foundation\Application;
use Illuminate\Http\Request;

define('LARAVEL_START', microtime(true));

if (file_exists($maintenance = __DIR__.'/../storage/framework/maintenance.php')) {
    require $maintenance;
}

require __DIR__.'/../vendor/autoload.php';

/** @var Application $app */
$app = require_once __DIR__.'/../bootstrap/app.php';

$app->handleRequest(Request::capture());
"#;
        let facts = parse_index_facts(
            source,
            FileId(4),
            EchoFileMode::PhpCompat,
            Some(Path::new("/project/public")),
        )
        .expect("source parses");

        assert!(facts.dependencies.iter().any(|dependency| {
            dependency.kind == DependencyKind::PhpUse
                && dependency.target == "Illuminate\\Http\\Request"
        }));
        let use_request_start = source
            .find("Illuminate\\Http\\Request")
            .expect("use request");
        assert!(facts.dependencies.iter().any(|dependency| {
            dependency.kind == DependencyKind::PhpUse
                && dependency.target == "Illuminate\\Http\\Request"
                && dependency.target_range
                    == TextRange::new(
                        use_request_start as u32,
                        (use_request_start + "Illuminate\\Http\\Request".len()) as u32,
                    )
        }));
        let phpdoc_application_start = source.find("@var Application").expect("phpdoc app") + 5;
        assert!(facts.references.iter().any(|reference| {
            reference.kind == ReferenceKind::ClassLike
                && reference.name == "Application"
                && reference.range
                    == TextRange::new(
                        phpdoc_application_start as u32,
                        (phpdoc_application_start + "Application".len()) as u32,
                    )
        }));
        let autoload_start = source
            .find("__DIR__.'/../vendor/autoload.php'")
            .expect("autoload");
        assert!(facts.references.iter().any(|reference| {
            reference.kind == ReferenceKind::FilePath
                && reference.name == "/project/public/../vendor/autoload.php"
                && reference.range
                    == TextRange::new(
                        autoload_start as u32,
                        (autoload_start + "__DIR__.'/../vendor/autoload.php'".len()) as u32,
                    )
        }));
        let request_call_start = source.find("Request::capture").expect("static call");
        assert!(facts.references.iter().any(|reference| {
            reference.kind == ReferenceKind::ClassLike
                && reference.name == "Request"
                && reference.range
                    == TextRange::new(
                        request_call_start as u32,
                        (request_call_start + "Request".len()) as u32,
                    )
        }));
        assert!(facts.references.iter().any(|reference| {
            reference.kind == ReferenceKind::StaticMethod
                && reference.name == "capture"
                && reference.qualifier.as_deref() == Some("Request")
        }));
        assert!(facts.references.iter().any(|reference| {
            reference.kind == ReferenceKind::Method
                && reference.name == "handleRequest"
                && reference.qualifier.as_deref() == Some("app")
        }));
        assert!(facts.declarations.iter().any(|symbol| {
            symbol.kind == SymbolKind::LocalVariable
                && symbol.name.text == "app"
                && symbol
                    .signature
                    .as_ref()
                    .is_some_and(|signature| signature.text == "Application")
        }));
    }

    #[test]
    fn indexes_required_php_files_from_laravel_fixture() {
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tests/php/112_laravel_bootstrap")
            .canonicalize()
            .expect("fixture root");
        let public_index = fixture_root.join("public/index.php");
        let public_source = std::fs::read_to_string(&public_index).expect("public source");

        let mut index = EchoIndex::new();
        let file_id = index.alloc_file_id();
        index.insert_file(IndexedFile {
            file_id,
            uri: Uri::from_file_path(&public_index).unwrap().to_string(),
            path: Some(public_index.clone()),
            version: None,
            mode: EchoFileMode::PhpCompat,
            content_hash: None,
        });

        let facts = parse_index_facts(
            &public_source,
            file_id,
            EchoFileMode::PhpCompat,
            public_index.parent(),
        )
        .expect("public source parses");
        index.update_file(file_id, facts);
        index_required_files(&mut index, file_id);

        let method = index
            .method_definition("Illuminate\\Foundation\\Application", "handleRequest")
            .expect("vendored handleRequest method");
        let method_file = index.file(method.file_id).expect("method file");

        assert_eq!(
            method_file.path.as_deref(),
            Some(
                fixture_root
                    .join("vendor/laravel/framework/src/Illuminate/Foundation/Application.php")
                    .as_path()
            )
        );
    }

    #[test]
    fn document_links_cover_whole_dir_path_expression() {
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tests/php/112_laravel_bootstrap")
            .canonicalize()
            .expect("fixture root");
        let public_index = fixture_root.join("public/index.php");
        let public_source = std::fs::read_to_string(&public_index).expect("public source");
        let autoload_expr = "__DIR__.'/../vendor/autoload.php'";
        let autoload_start = public_source.find(autoload_expr).expect("autoload expr");
        let maintenance_expr = "__DIR__.'/../storage/framework/maintenance.php'";
        let maintenance_start = public_source
            .find(maintenance_expr)
            .expect("maintenance expr");
        let mut index = EchoIndex::new();
        let file_id = index.alloc_file_id();
        index.insert_file(IndexedFile {
            file_id,
            uri: Uri::from_file_path(&public_index).unwrap().to_string(),
            path: Some(public_index.clone()),
            version: None,
            mode: EchoFileMode::PhpCompat,
            content_hash: None,
        });
        let facts = parse_index_facts(
            &public_source,
            file_id,
            EchoFileMode::PhpCompat,
            public_index.parent(),
        )
        .expect("public source parses");
        index.update_file(file_id, facts);

        let dependencies = index.dependencies(DependencyQuery::in_file(file_id));
        let references = index.references(ReferenceQuery::in_file(file_id));
        let links =
            document_links_for_paths(&Rope::from_str(&public_source), &dependencies, &references);
        let link = links
            .iter()
            .find(|link| {
                link.target.as_ref()
                    == Some(&Uri::from_file_path(fixture_root.join("vendor/autoload.php")).unwrap())
            })
            .expect("autoload document link");
        let autoload_target_links = links
            .iter()
            .filter(|link| {
                link.target.as_ref()
                    == Some(&Uri::from_file_path(fixture_root.join("vendor/autoload.php")).unwrap())
            })
            .collect::<Vec<_>>();

        assert_eq!(
            link.range,
            range_to_lsp_range(
                &Rope::from_str(&public_source),
                TextRange::new(
                    autoload_start as u32,
                    (autoload_start + autoload_expr.len()) as u32,
                ),
            )
        );
        assert_eq!(autoload_target_links.len(), 1);

        let maintenance_target =
            Uri::from_file_path(fixture_root.join("storage/framework/maintenance.php")).unwrap();
        assert!(
            links
                .iter()
                .all(|link| link.target.as_ref() != Some(&maintenance_target)),
            "missing maintenance file should not produce a document link"
        );

        assert!(references.iter().any(|reference| {
            reference.kind == ReferenceKind::FilePath
                && reference.range
                    == TextRange::new(
                        maintenance_start as u32,
                        (maintenance_start + maintenance_expr.len()) as u32,
                    )
        }));
    }
}
