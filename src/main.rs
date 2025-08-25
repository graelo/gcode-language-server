use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use regex::Regex;
use serde::Deserialize;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug, Deserialize)]
struct Flavor {
    flavor: FlavorMeta,
    commands: Option<Vec<CommandDef>>,
}

#[derive(Debug, Deserialize)]
struct FlavorMeta {
    name: String,
    version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CommandDef {
    name: String,
    pattern: Option<String>,
    description_short: Option<String>,
    description_long: Option<String>,
}

struct Backend {
    client: Client,
    commands: Arc<HashMap<String, CommandDef>>,
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
        if let Some(pos) = params
            .text_document_position_params
            .position
            .line
            .checked_add(0)
        {
            // naive: extract the word at position by asking the client to open the file isn't available here.
            // For MVP, we will return hover if the word under the cursor matches a known command from our map by scanning the whole document (not ideal but simple).
        }

        // Read document content via client (not available synchronously), skip and return None.
        // Instead, return None to be safe in this scaffold.
        Ok(None)
    }

    async fn completion(&self, _: CompletionParams) -> LspResult<Option<CompletionResponse>> {
        Ok(None)
    }

    // ... other methods no-op for now
    async fn did_open(&self, _: DidOpenTextDocumentParams) {}

    async fn did_change(&self, _: DidChangeTextDocumentParams) {}
}

fn load_sample_flavor() -> Result<HashMap<String, CommandDef>> {
    let sample = include_str!("../docs/work/samples/prusa.gcode-flavor.toml");
    let flavor: Flavor = toml::from_str(sample)?;
    let mut map = HashMap::new();
    if let Some(cmds) = flavor.commands {
        for c in cmds {
            map.insert(c.name.clone(), c);
        }
    }
    Ok(map)
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    env_logger::init();

    let commands = load_sample_flavor()?;
    let commands = Arc::new(commands);

    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());

    let (service, socket) = LspService::build(|client| Backend {
        client,
        commands: commands.clone(),
    })
    .finish();
    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}
