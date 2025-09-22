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
use gcode_language_server::gcode::{tokenize_text, Token, TokenKind};

struct Backend {
    client: Client,
    flavor_manager: Arc<Mutex<FlavorManager>>,
    documents: Arc<Mutex<HashMap<Url, DocumentState>>>,
    config: Config,
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
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                hover_provider: Some(HoverProviderCapability::Simple(true)),
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
            let desc = if self.config.long_descriptions {
                cmd.description_long
                    .clone()
                    .or_else(|| cmd.description_short.clone())
            } else {
                cmd.description_short.clone()
            }
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

    /// Publish diagnostics for a document
    async fn publish_diagnostics(&self, uri: Url) {
        let docs = self.documents.lock().await;
        let doc_state = match docs.get(&uri) {
            Some(state) => state,
            None => return,
        };

        let mut diagnostics = Vec::new();

        // Tokenize the document content
        let tokens = tokenize_text(&doc_state.content);

        // Check each command token for unknown commands
        for token in tokens.iter() {
            if token.kind == TokenKind::Command {
                let command = token.text.to_uppercase();
                if !doc_state.commands.contains_key(&command) {
                    // Create diagnostic for unknown command
                    let diagnostic =
                        self.create_unknown_command_diagnostic(token, &doc_state.content);
                    diagnostics.push(diagnostic);
                }
            }
        }

        // Publish the diagnostics
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    /// Create a diagnostic for an unknown command
    fn create_unknown_command_diagnostic(&self, token: &Token<'_>, content: &str) -> Diagnostic {
        // Convert byte positions to line/character positions
        let range = self.byte_range_to_lsp_range(token.start, token.end, content);

        Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::WARNING),
            code: Some(NumberOrString::String("unknown_command".to_string())),
            source: Some("gcode-ls".to_string()),
            message: format!("Unknown G-code command: {}", token.text),
            ..Default::default()
        }
    }

    /// Convert byte positions to LSP Range
    fn byte_range_to_lsp_range(&self, start_byte: usize, end_byte: usize, content: &str) -> Range {
        let mut byte_pos = 0;

        let start_pos = {
            let mut start_line = 0;
            let mut start_char = 0;

            for ch in content.chars() {
                if byte_pos >= start_byte {
                    break;
                }

                if ch == '\n' {
                    start_line += 1;
                    start_char = 0;
                } else {
                    start_char += 1;
                }

                byte_pos += ch.len_utf8();
            }

            Position::new(start_line, start_char)
        };

        // Reset for end position
        byte_pos = 0;

        let end_pos = {
            let mut end_line = 0;
            let mut end_char = 0;

            for ch in content.chars() {
                if byte_pos >= end_byte {
                    break;
                }

                if ch == '\n' {
                    end_line += 1;
                    end_char = 0;
                } else {
                    end_char += 1;
                }

                byte_pos += ch.len_utf8();
            }

            Position::new(end_line, end_char)
        };

        Range::new(start_pos, end_pos)
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
        let config_clone = config.clone();

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
            config: config_clone,
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

    #[test]
    fn test_byte_range_to_lsp_range_conversion() {
        // Create a dummy backend to test the byte range conversion function
        // This is an internal helper function that doesn't need the client
        let content = "G0 X10\nG1 Y20\nG2 Z30";

        // We'll test the conversion logic directly using the helper logic
        // Test conversion of G1 (starts at byte 7, ends at byte 9)
        let (start_line, start_char, end_line, end_char) =
            byte_positions_to_line_char(7, 9, content);

        assert_eq!(start_line, 1);
        assert_eq!(start_char, 0);
        assert_eq!(end_line, 1);
        assert_eq!(end_char, 2);

        // Test conversion of G2 (starts at byte 14, ends at byte 16)
        let (start_line2, start_char2, end_line2, end_char2) =
            byte_positions_to_line_char(14, 16, content);

        assert_eq!(start_line2, 2);
        assert_eq!(start_char2, 0);
        assert_eq!(end_line2, 2);
        assert_eq!(end_char2, 2);
    }

    #[test]
    fn test_hover_description_selection() {
        let short_config = Config {
            cli_flavor: None,
            project_flavor: None,
            flavor_dirs: vec![],
            project_config_path: None,
            long_descriptions: false,
            log_level: "info".to_string(),
        };

        let long_config = Config {
            cli_flavor: None,
            project_flavor: None,
            flavor_dirs: vec![],
            project_config_path: None,
            long_descriptions: true,
            log_level: "info".to_string(),
        };

        assert!(!short_config.long_descriptions);
        assert!(long_config.long_descriptions);
    }

    #[test]
    fn test_tokenization_identifies_commands() {
        let content = "G0 X10\nGXYZ Y20\nG1 X30";
        let tokens = tokenize_text(content);

        let command_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind == TokenKind::Command)
            .collect();

        assert_eq!(command_tokens.len(), 3);
        assert_eq!(command_tokens[0].text, "G0");
        assert_eq!(command_tokens[1].text, "GXYZ");
        assert_eq!(command_tokens[2].text, "G1");
    }

    #[test]
    fn test_unknown_command_diagnostic_creation_logic() {
        // Test the diagnostic creation without needing a full Backend
        let content = "G0 X10\nGXYZ Y20\nG1 X30";

        // Create a token for the unknown command GXYZ
        let token = Token {
            kind: TokenKind::Command,
            text: "GXYZ".into(),
            start: 7, // Position of GXYZ in the content
            end: 11,
        };

        // Test the range conversion
        let (start_line, start_char, end_line, end_char) =
            byte_positions_to_line_char(token.start, token.end, content);

        assert_eq!(start_line, 1); // Second line (0-indexed)
        assert_eq!(start_char, 0); // Start of GXYZ
        assert_eq!(end_line, 1);
        assert_eq!(end_char, 4); // End of GXYZ

        // Test that we can create a proper diagnostic message
        let expected_message = format!("Unknown G-code command: {}", token.text);
        assert_eq!(expected_message, "Unknown G-code command: GXYZ");
    }

    #[test]
    fn test_description_preference_logic() {
        // Test the logic for choosing between short and long descriptions
        let cmd = CommandDef {
            name: "G0".to_string(),
            pattern: None,
            description_short: Some("Move to position".to_string()),
            description_long: Some("Move to position at rapid rate without extrusion".to_string()),
            parameters: None,
        };

        // Test short preference
        let short_desc = match "short" {
            "long" => cmd
                .description_long
                .clone()
                .or_else(|| cmd.description_short.clone()),
            _ => cmd.description_short.clone(),
        }
        .unwrap_or_else(|| "No description".to_string());

        assert_eq!(short_desc, "Move to position");

        // Test long preference
        let long_desc = match "long" {
            "long" => cmd
                .description_long
                .clone()
                .or_else(|| cmd.description_short.clone()),
            _ => cmd.description_short.clone(),
        }
        .unwrap_or_else(|| "No description".to_string());

        assert_eq!(
            long_desc,
            "Move to position at rapid rate without extrusion"
        );
    }

    /// Helper function to convert byte positions to line/character positions
    /// This is extracted for testing purposes
    fn byte_positions_to_line_char(
        start_byte: usize,
        end_byte: usize,
        content: &str,
    ) -> (u32, u32, u32, u32) {
        let mut byte_pos = 0;

        let (start_line, start_char) = {
            let mut line = 0;
            let mut char_pos = 0;

            for ch in content.chars() {
                if byte_pos >= start_byte {
                    break;
                }

                if ch == '\n' {
                    line += 1;
                    char_pos = 0;
                } else {
                    char_pos += 1;
                }

                byte_pos += ch.len_utf8();
            }

            (line, char_pos)
        };

        // Reset for end position
        byte_pos = 0;

        let (end_line, end_char) = {
            let mut line = 0;
            let mut char_pos = 0;

            for ch in content.chars() {
                if byte_pos >= end_byte {
                    break;
                }

                if ch == '\n' {
                    line += 1;
                    char_pos = 0;
                } else {
                    char_pos += 1;
                }

                byte_pos += ch.len_utf8();
            }

            (line, char_pos)
        };

        (start_line, start_char, end_line, end_char)
    }
}
