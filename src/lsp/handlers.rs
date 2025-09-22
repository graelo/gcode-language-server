use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::*;

use crate::flavor::schema::ParameterType;
use crate::lsp::backend::Backend;
use crate::lsp::document::DocumentState;
use crate::validation::engine::validate_document;

/// Trait for handling hover requests
#[tower_lsp::async_trait]
pub trait HandleHover {
    async fn handle_hover(&self, params: HoverParams) -> LspResult<Option<Hover>>;
}

/// Trait for handling completion requests
#[tower_lsp::async_trait]
pub trait HandleCompletion {
    async fn handle_completion(
        &self,
        params: CompletionParams,
    ) -> LspResult<Option<CompletionResponse>>;
}

/// Trait for handling diagnostics
#[tower_lsp::async_trait]
pub trait HandleDiagnostics {
    async fn create_document_state(&self, content: String) -> DocumentState;
    async fn publish_diagnostics(&self, uri: Url);
    fn create_lsp_diagnostic(
        &self,
        validation_diagnostic: crate::validation::engine::Diagnostic,
    ) -> tower_lsp::lsp_types::Diagnostic;
}

#[tower_lsp::async_trait]
impl HandleHover for Backend {
    async fn handle_hover(&self, params: HoverParams) -> LspResult<Option<Hover>> {
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

        // Use the new flavor registry instead of legacy cached commands
        let registry = self.flavor_registry.lock().await;
        if let Some(cmd) = registry.get_command(&token_up) {
            // Always show rich information: prefer long description, fallback to short
            let desc = cmd
                .description_long
                .clone()
                .or_else(|| cmd.description_short.clone())
                .unwrap_or_else(|| "No description".to_string());

            // Enhance hover with parameter information
            let mut hover_text = format!("**{}**\n\n{}", token_up, desc);

            if let Some(parameters) = &cmd.parameters {
                if !parameters.is_empty() {
                    hover_text.push_str("\n\n**Parameters:**");
                    for param in parameters {
                        hover_text.push_str(&format!(
                            "\n- `{}`: {} ({:?}{})",
                            param.name,
                            param.description,
                            param.param_type,
                            if param.required {
                                ", required"
                            } else {
                                ", optional"
                            }
                        ));
                    }
                }
            }

            let m = MarkupContent {
                kind: MarkupKind::Markdown,
                value: hover_text,
            };
            return Ok(Some(Hover {
                contents: HoverContents::Markup(m),
                range: None,
            }));
        }

        Ok(None)
    }
}

#[tower_lsp::async_trait]
impl HandleCompletion for Backend {
    async fn handle_completion(
        &self,
        params: CompletionParams,
    ) -> LspResult<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;

        let docs = self.documents.lock().await;
        let doc_state = match docs.get(&uri) {
            Some(state) => state,
            None => return Ok(None),
        };

        let line_idx = pos.line as usize;
        let line = doc_state.content.lines().nth(line_idx).unwrap_or("");
        let char_idx = pos.character as usize;

        // Parse the line to understand context
        let words: Vec<&str> = line.split_whitespace().collect();
        let mut completions = Vec::new();

        // Get all commands from the flavor registry
        let registry = self.flavor_registry.lock().await;
        let active_flavor = match registry.get_active_flavor() {
            Some(flavor) => flavor,
            None => return Ok(None),
        };

        // Determine if we're completing a command or parameters
        let line_up_to_cursor = &line[..char_idx.min(line.len())];
        let is_after_space = line_up_to_cursor.ends_with(' ');

        if words.is_empty() || (words.len() == 1 && !is_after_space) {
            // Completing a command
            let current_word = if words.is_empty() { "" } else { words[0] }.to_uppercase();

            for (command_name, command_def) in &active_flavor.commands {
                if command_name.starts_with(&current_word) {
                    // Use short description for completion detail (concise summary)
                    let detail = command_def
                        .description_short
                        .clone()
                        .unwrap_or_else(|| "G-code command".to_string());

                    // Use long description for documentation (comprehensive info)
                    let mut documentation = command_def
                        .description_long
                        .clone()
                        .or_else(|| command_def.description_short.clone())
                        .unwrap_or_else(|| "G-code command".to_string());

                    // Add parameter information to documentation
                    if let Some(parameters) = &command_def.parameters {
                        if !parameters.is_empty() {
                            documentation.push_str("\n\n**Parameters:**");
                            for param in parameters {
                                documentation.push_str(&format!(
                                    "\n- `{}`: {} ({:?}{})",
                                    param.name,
                                    param.description,
                                    param.param_type,
                                    if param.required {
                                        ", required"
                                    } else {
                                        ", optional"
                                    }
                                ));
                            }
                        }
                    }

                    completions.push(CompletionItem {
                        label: command_name.clone(),
                        kind: Some(CompletionItemKind::KEYWORD),
                        detail: Some(detail),
                        documentation: Some(Documentation::MarkupContent(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: documentation,
                        })),
                        ..Default::default()
                    });
                }
            }
        } else if words.len() >= 1 && is_after_space {
            // Completing parameters for a command (cursor is after a space following the command)
            let command_name = words[0].to_uppercase();
            if let Some(command_def) = active_flavor.commands.get(&command_name) {
                if let Some(parameters) = &command_def.parameters {
                    // Parse existing parameters to avoid duplicates
                    let mut existing_params = std::collections::HashSet::new();
                    for word in &words[1..] {
                        if let Some(param_name) = word.split(&['=', ':']).next() {
                            existing_params.insert(param_name.to_uppercase());
                        }
                    }

                    // Add completions for parameters not yet used
                    for param in parameters {
                        let param_upper = param.name.to_uppercase();
                        if !existing_params.contains(&param_upper) {
                            completions.push(CompletionItem {
                                label: param.name.clone(),
                                kind: Some(CompletionItemKind::PROPERTY),
                                detail: Some(format!("{:?}", param.param_type)),
                                documentation: Some(Documentation::String(
                                    param.description.clone(),
                                )),
                                sort_text: Some(format!(
                                    "{}{}",
                                    if param.required { "0" } else { "1" },
                                    param.name
                                )),
                                insert_text: Some(match param.param_type {
                                    ParameterType::Float => format!("{}0.0", param.name),
                                    ParameterType::Int => format!("{}0", param.name),
                                    ParameterType::Bool => param.name.clone(),
                                    ParameterType::String => format!("{}\"\"", param.name),
                                }),
                                insert_text_format: Some(InsertTextFormat::SNIPPET),
                                preselect: Some(param.required),
                                filter_text: Some(param.name.clone()),
                                ..Default::default()
                            });
                        }
                    }
                }
            }
        }

        if completions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(CompletionResponse::Array(completions)))
        }
    }
}

#[tower_lsp::async_trait]
impl HandleDiagnostics for Backend {
    /// Create a new document state, detecting flavor and caching commands
    async fn create_document_state(&self, content: String) -> DocumentState {
        let mut flavor_registry = self.flavor_registry.lock().await;

        // Try to detect flavor from modeline (highest priority)
        let modeline_flavor = flavor_registry.detect_modeline_flavor(&content);

        // Set the appropriate flavor
        let flavor_name = if let Some(ref name) = modeline_flavor {
            // Try to set the detected flavor
            if flavor_registry.set_active_flavor(name) {
                modeline_flavor.clone()
            } else {
                // Fallback if detected flavor doesn't exist
                flavor_registry.get_active_flavor().map(|f| f.name.clone())
            }
        } else {
            // Use current active flavor or ensure we have one
            flavor_registry.get_active_flavor().map(|f| f.name.clone())
        };

        DocumentState {
            content,
            flavor_name,
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

        // Use enhanced validation with parameter checking
        let flavor_registry = self.flavor_registry.lock().await;
        let validation_result = validate_document(&doc_state.content, &flavor_registry);

        // Convert validation results to LSP diagnostics
        for validation_diagnostic in validation_result.diagnostics {
            let lsp_diagnostic = self.create_lsp_diagnostic(validation_diagnostic);
            diagnostics.push(lsp_diagnostic);
        }

        // Publish the diagnostics
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    fn create_lsp_diagnostic(
        &self,
        validation_diagnostic: crate::validation::engine::Diagnostic,
    ) -> tower_lsp::lsp_types::Diagnostic {
        use crate::validation::engine::Severity;

        let severity = match validation_diagnostic.severity {
            Severity::Error => DiagnosticSeverity::ERROR,
            Severity::Warning => DiagnosticSeverity::WARNING,
            Severity::Info => DiagnosticSeverity::INFORMATION,
        };

        tower_lsp::lsp_types::Diagnostic::new(
            Range::new(
                Position::new((validation_diagnostic.line - 1) as u32, 0),
                Position::new((validation_diagnostic.line - 1) as u32, 100), // Arbitrary end position
            ),
            Some(severity),
            None,
            Some("gcode-ls".to_string()),
            validation_diagnostic.message,
            None,
            None,
        )
    }
}
