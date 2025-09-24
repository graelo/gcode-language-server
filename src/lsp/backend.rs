use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::flavor::registry::FlavorRegistry;
use crate::lsp::document::DocumentState;
use crate::lsp::handlers::{
    HandleCompletion, HandleDiagnostics, HandleDocumentSymbol, HandleHover,
};
use crate::Config;

/// The main LSP backend that holds state and implements the Language Server Protocol
pub struct Backend {
    pub client: Client,
    pub flavor_registry: Arc<Mutex<FlavorRegistry>>,
    pub documents: Arc<Mutex<HashMap<Url, DocumentState>>>,
    pub config: Config,
}

impl Backend {
    pub fn new(client: Client, config: Config, flavor_registry: FlavorRegistry) -> Self {
        let flavor_registry = Arc::new(Mutex::new(flavor_registry));

        Self {
            client,
            flavor_registry,
            documents: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(
        &self,
        _: InitializeParams,
    ) -> tower_lsp::jsonrpc::Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![
                        "G".to_string(),
                        "M".to_string(),
                        "T".to_string(),
                    ]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                document_symbol_provider: Some(OneOf::Left(true)),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "gcode-language-server initialized")
            .await;
    }

    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
        Ok(())
    }

    async fn hover(&self, params: HoverParams) -> tower_lsp::jsonrpc::Result<Option<Hover>> {
        self.handle_hover(params).await
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<CompletionResponse>> {
        self.handle_completion(params).await
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> tower_lsp::jsonrpc::Result<Option<DocumentSymbolResponse>> {
        self.handle_document_symbol(params).await
    }

    // Store opened documents for hover/diagnostics
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let content = params.text_document.text;

        // Create document state with flavor detection
        let doc_state = self.create_document_state(content).await;

        let mut docs = self.documents.lock().await;
        docs.insert(uri.clone(), doc_state);
        drop(docs); // Release the lock before calling publish_diagnostics

        // Publish diagnostics for the opened document
        self.publish_diagnostics(uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        if let Some(change) = params.content_changes.into_iter().last() {
            // Create new document state with updated content
            let doc_state = self.create_document_state(change.text).await;

            let mut docs = self.documents.lock().await;
            docs.insert(uri.clone(), doc_state);
            drop(docs); // Release the lock before calling publish_diagnostics

            // Publish updated diagnostics
            self.publish_diagnostics(uri).await;
        }
    }
}
