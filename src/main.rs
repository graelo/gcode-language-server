use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use std::thread;
use std::time::Duration;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use gcode_language_server::config::Config;
use gcode_language_server::flavor::{CommandDef, FlavorManager};

struct Backend {
    client: Client,
    flavor_manager: Arc<Mutex<FlavorManager>>,
    documents: Arc<Mutex<HashMap<Url, DocumentState>>>,
}

/// State for each open document
#[derive(Debug)]
struct DocumentState {
    content: String,
    #[allow(dead_code)]
    flavor_name: Option<String>, // Detected from modeline or default
    commands: HashMap<String, CommandDef>, // Cached command lookup
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> LspResult<InitializeResult> {
        Ok(InitializeResult::default())
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "gcode-language-server initialized")
            .await;
    }

    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }

    async fn hover(&self, params: HoverParams) -> LspResult<Option<Hover>> {
        let tdpp = params.text_document_position_params;
        let uri = tdpp.text_document.uri;
        let pos = tdpp.position;

        let docs = self.documents.lock().await;
        let doc_state = match docs.get(&uri) {
            Some(state) => state,
            None => return Ok(None),
        };

        let line_idx = pos.line as usize;
        let line = doc_state.content.lines().nth(line_idx).unwrap_or("");
        let char_idx = pos.character as usize;

        // Find token under cursor (alphanumeric)
        let mut start = char_idx;
        while start > 0 {
            let c = line.chars().nth(start - 1).unwrap_or(' ');
            if c.is_alphanumeric() {
                start -= 1;
            } else {
                break;
            }
        }
        let mut end = char_idx;
        while end < line.len() {
            let c = line.chars().nth(end).unwrap_or(' ');
            if c.is_alphanumeric() {
                end += 1;
            } else {
                break;
            }
        }

        if start >= end {
            return Ok(None);
        }

        let token: String = line.chars().skip(start).take(end - start).collect();
        let token_up = token.to_uppercase();

        if let Some(cmd) = doc_state.commands.get(&token_up) {
            let desc = cmd
                .description_short
                .clone()
                .unwrap_or_else(|| "No description".to_string());
            let m = MarkupContent {
                kind: MarkupKind::Markdown,
                value: desc,
            };
            return Ok(Some(Hover {
                contents: HoverContents::Markup(m),
                range: None,
            }));
        }

        Ok(None)
    }

    async fn completion(&self, _: CompletionParams) -> LspResult<Option<CompletionResponse>> {
        Ok(None)
    }

    // store opened documents for hover/diagnostics
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let content = params.text_document.text;

        // Create document state with flavor detection
        let doc_state = self.create_document_state(content).await;

        let mut docs = self.documents.lock().await;
        docs.insert(uri, doc_state);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.into_iter().last() {
            // Create new document state with updated content
            let doc_state = self.create_document_state(change.text).await;

            let mut docs = self.documents.lock().await;
            docs.insert(uri, doc_state);
        }
    }
}

impl Backend {
    /// Create a new document state, detecting flavor and caching commands
    async fn create_document_state(&self, content: String) -> DocumentState {
        let flavor_manager = self.flavor_manager.lock().await;

        // Try to detect flavor from modeline (highest priority)
        let modeline_flavor = flavor_manager.detect_modeline_flavor(&content);

        // Get the appropriate flavor
        let loaded_flavor = if let Some(ref name) = modeline_flavor {
            flavor_manager.get_flavor(name).await
        } else {
            // Use effective default (considers CLI/project config)
            flavor_manager.get_effective_default_flavor().await
        };

        // Create command lookup map
        let commands = if let Some(loaded_flavor) = loaded_flavor {
            flavor_manager.flavor_to_command_map(&loaded_flavor.flavor)
        } else {
            HashMap::new()
        };

        DocumentState {
            content,
            #[allow(dead_code)]
            flavor_name: modeline_flavor,
            commands,
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    env_logger::init();

    // Parse configuration from command line and environment
    let config = Config::from_args_and_env()?;

    // Create and initialize flavor manager
    let flavor_manager = FlavorManager::new(&config)?;

    let documents = Arc::new(Mutex::new(HashMap::new()));

    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());

    // If running under the integration test, exit after a short delay so the test can read stdout to EOF.
    if std::env::var("GCODE_LS_TEST_EXIT").as_deref() == Ok("1") {
        thread::spawn(|| {
            thread::sleep(Duration::from_secs(1));
            std::process::exit(0);
        });
    }

    let (service, socket) = LspService::build(|client| {
        let flavor_manager_arc = Arc::new(Mutex::new(flavor_manager));

        // Initialize the flavor manager with the client in a background task
        let flavor_manager_clone = flavor_manager_arc.clone();
        let client_clone = client.clone();
        tokio::spawn(async move {
            let mut fm = flavor_manager_clone.lock().await;
            if let Err(e) = fm.initialize(Some(client_clone.clone())).await {
                client_clone
                    .log_message(
                        MessageType::ERROR,
                        format!("Failed to initialize flavor manager: {}", e),
                    )
                    .await;
            }
        });

        Backend {
            client,
            flavor_manager: flavor_manager_arc,
            documents: documents.clone(),
        }
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn flavor_manager_loads_builtin_flavors() {
        let mut flavor_manager =
            FlavorManager::with_default_config().expect("create flavor manager");
        flavor_manager
            .initialize(None)
            .await
            .expect("initialize flavor manager");

        let prusa_flavor = flavor_manager.get_flavor("prusa").await;
        assert!(prusa_flavor.is_some());

        let flavor = prusa_flavor.unwrap();
        let commands = flavor_manager.flavor_to_command_map(&flavor.flavor);
        assert!(commands.contains_key("G0"));
        assert!(commands.contains_key("G1"));
    }
}
