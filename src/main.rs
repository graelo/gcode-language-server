use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use serde::Deserialize;
use std::thread;
use std::time::Duration;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Flavor {
    flavor: FlavorMeta,
    commands: Option<Vec<CommandDef>>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct FlavorMeta {
    name: String,
    version: Option<String>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize)]
struct CommandDef {
    name: String,
    pattern: Option<String>,
    description_short: Option<String>,
    description_long: Option<String>,
}

struct Backend {
    client: Client,
    commands: Arc<HashMap<String, CommandDef>>,
    documents: Arc<Mutex<HashMap<Url, String>>>,
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
        let text = match docs.get(&uri) {
            Some(t) => t,
            None => return Ok(None),
        };

        let line_idx = pos.line as usize;
        let line = text.lines().nth(line_idx).unwrap_or("");
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

        if let Some(cmd) = self.commands.get(&token_up) {
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
        let text = params.text_document.text;
        let mut docs = self.documents.lock().await;
        docs.insert(uri, text);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.into_iter().last() {
            let mut docs = self.documents.lock().await;
            docs.insert(uri, change.text);
        }
    }
}

fn load_sample_flavor() -> Result<HashMap<String, CommandDef>> {
    let sample = include_str!("../docs/work/samples/prusa.gcode-flavor.toml");
    let flavor: Flavor = toml::from_str(sample)?;
    let mut map = HashMap::new();
    if let Some(cmds) = flavor.commands {
        for c in cmds {
            map.insert(c.name.to_uppercase(), c);
        }
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_flavor_contains_common_commands() {
        let map = load_sample_flavor().expect("load flavor");
        assert!(map.contains_key("G0"));
        assert!(map.contains_key("G1"));
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    env_logger::init();
    // simple CLI flag for quick smoke checks
    let mut args = std::env::args();
    // skip executable name
    args.next();
    if let Some(first) = args.next() {
        if first == "--version" {
            println!("{}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
    }

    let commands = load_sample_flavor()?;
    let commands = Arc::new(commands);
    let documents = Arc::new(Mutex::new(HashMap::new()));

    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
    // If running under the integration test, exit after a short delay so the test can read stdout to EOF.
    if std::env::var("GCODE_LS_TEST_EXIT").as_deref() == Ok("1") {
        thread::spawn(|| {
            thread::sleep(Duration::from_secs(1));
            std::process::exit(0);
        });
    }

    let (service, socket) = LspService::build(|client| Backend {
        client,
        commands: commands.clone(),
        documents: documents.clone(),
    })
    .finish();
    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}
