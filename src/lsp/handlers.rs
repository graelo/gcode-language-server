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

/// Trait for handling document symbols
#[tower_lsp::async_trait]
pub trait HandleDocumentSymbol {
    async fn handle_document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> LspResult<Option<DocumentSymbolResponse>>;
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
        } else if !words.is_empty() && is_after_space {
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

#[tower_lsp::async_trait]
impl HandleDocumentSymbol for Backend {
    async fn handle_document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> LspResult<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;

        let docs = self.documents.lock().await;
        let doc_state = match docs.get(&uri) {
            Some(state) => state,
            None => return Ok(None),
        };

        // Get flavor registry for enhanced symbol details
        let flavor_registry = self.flavor_registry.lock().await;

        let mut symbols = Vec::new();

        for (line_idx, line) in doc_state.content.lines().enumerate() {
            let parsed = crate::parser::parse_line(line);

            if let crate::parser::ParsedLine::Command(command) = parsed {
                // Generate basic symbol name (command + first 3 parameters)
                let symbol_name = if command.parameters.is_empty() {
                    command.name.clone()
                } else {
                    let params: Vec<String> = command
                        .parameters
                        .iter()
                        .take(3) // Limit to first 3 parameters
                        .map(|p| format!("{}{}", p.letter, p.value))
                        .collect();
                    format!("{} {}", command.name, params.join(" "))
                };

                // Enhanced symbol detail using flavor registry
                let symbol_detail = flavor_registry.get_command(&command.name).map(|cmd_def| {
                    let mut detail = cmd_def
                        .description_short
                        .clone()
                        .unwrap_or_else(|| "G-code command".to_string());

                    // Add parameter documentation for parameters present in this command
                    if !command.parameters.is_empty() {
                        if let Some(flavor_params) = &cmd_def.parameters {
                            let mut param_docs = Vec::new();

                            // Match actual parameters with flavor definitions
                            for param in &command.parameters {
                                let param_upper = param.letter.to_uppercase().to_string();
                                if let Some(flavor_param) = flavor_params
                                    .iter()
                                    .find(|fp| fp.name.to_uppercase() == param_upper)
                                {
                                    param_docs.push(format!(
                                        "{}: {}",
                                        param.letter, flavor_param.description
                                    ));
                                }
                            }

                            if !param_docs.is_empty() {
                                detail.push_str(" | ");
                                detail.push_str(&param_docs.join(", "));
                            }
                        }
                    }

                    detail
                });

                let symbol_kind = match command.name.chars().next() {
                    Some('G') => SymbolKind::FUNCTION,
                    Some('M') => SymbolKind::PROPERTY,
                    Some('T') => SymbolKind::VARIABLE,
                    _ => SymbolKind::FUNCTION,
                };

                let range = Range::new(
                    Position::new(line_idx as u32, 0),
                    Position::new(line_idx as u32, line.len() as u32),
                );

                let selection_range = Range::new(
                    Position::new(line_idx as u32, 0),
                    Position::new(line_idx as u32, command.name.len() as u32),
                );

                let symbol = DocumentSymbol {
                    name: symbol_name,
                    detail: symbol_detail,
                    kind: symbol_kind,
                    tags: None,
                    #[allow(deprecated)]
                    deprecated: Some(false), // Required by tower-lsp 0.20, use tags instead in future versions
                    range,
                    selection_range,
                    children: None,
                };
                symbols.push(symbol);
            }
        }

        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{parse_line, Command, Parameter};

    #[test]
    fn test_symbol_name_generation() {
        // Test simple G-code with few parameters
        let mut command = Command {
            name: "G1".to_string(),
            parameters: vec![
                Parameter {
                    letter: 'X',
                    value: "10".to_string(),
                },
                Parameter {
                    letter: 'Y',
                    value: "20".to_string(),
                },
            ],
            comment: None,
        };

        let name = format!(
            "{} {}",
            command.name,
            command
                .parameters
                .iter()
                .take(3)
                .map(|p| format!("{}{}", p.letter, p.value))
                .collect::<Vec<_>>()
                .join(" ")
        );

        assert_eq!(name, "G1 X10 Y20");

        // Test with many parameters - should limit to first 3
        command.parameters = vec![
            Parameter {
                letter: 'X',
                value: "10".to_string(),
            },
            Parameter {
                letter: 'Y',
                value: "20".to_string(),
            },
            Parameter {
                letter: 'Z',
                value: "0.3".to_string(),
            },
            Parameter {
                letter: 'E',
                value: "5.5".to_string(),
            },
            Parameter {
                letter: 'F',
                value: "1500".to_string(),
            },
        ];

        let name = format!(
            "{} {}",
            command.name,
            command
                .parameters
                .iter()
                .take(3)
                .map(|p| format!("{}{}", p.letter, p.value))
                .collect::<Vec<_>>()
                .join(" ")
        );

        assert_eq!(name, "G1 X10 Y20 Z0.3");
    }

    #[test]
    fn test_symbol_kind_mapping() {
        // Test G-code -> FUNCTION
        let g_command = Command {
            name: "G1".to_string(),
            parameters: vec![],
            comment: None,
        };
        let kind = match g_command.name.chars().next().unwrap() {
            'G' => SymbolKind::FUNCTION,
            'M' => SymbolKind::PROPERTY,
            'T' => SymbolKind::VARIABLE,
            _ => SymbolKind::CONSTANT,
        };
        assert_eq!(kind, SymbolKind::FUNCTION);

        // Test M-code -> PROPERTY
        let m_command = Command {
            name: "M104".to_string(),
            parameters: vec![],
            comment: None,
        };
        let kind = match m_command.name.chars().next().unwrap() {
            'G' => SymbolKind::FUNCTION,
            'M' => SymbolKind::PROPERTY,
            'T' => SymbolKind::VARIABLE,
            _ => SymbolKind::CONSTANT,
        };
        assert_eq!(kind, SymbolKind::PROPERTY);

        // Test T-code -> VARIABLE
        let t_command = Command {
            name: "T1".to_string(),
            parameters: vec![],
            comment: None,
        };
        let kind = match t_command.name.chars().next().unwrap() {
            'G' => SymbolKind::FUNCTION,
            'M' => SymbolKind::PROPERTY,
            'T' => SymbolKind::VARIABLE,
            _ => SymbolKind::CONSTANT,
        };
        assert_eq!(kind, SymbolKind::VARIABLE);
    }

    #[test]
    fn test_parse_and_symbol_integration() {
        // Test that we can parse various G-code lines and extract symbol info
        let test_cases = vec![
            ("G28", "G28"),
            ("G1 X10 Y20", "G1 X10 Y20"),
            ("M104 S200", "M104 S200"),
            ("T1", "T1"),
            ("G1 X10 Y20 Z0.3 E5 F1500 S100", "G1 X10 Y20 Z0.3"), // Parameter limiting
        ];

        for (input, expected_name) in test_cases {
            let parsed = parse_line(input);
            if let crate::parser::ParsedLine::Command(cmd) = parsed {
                let symbol_name = if cmd.parameters.is_empty() {
                    cmd.name.clone()
                } else {
                    format!(
                        "{} {}",
                        cmd.name,
                        cmd.parameters
                            .iter()
                            .take(3)
                            .map(|p| format!("{}{}", p.letter, p.value))
                            .collect::<Vec<_>>()
                            .join(" ")
                    )
                };
                assert_eq!(symbol_name, expected_name, "Failed for input: {}", input);
            } else {
                panic!("Expected command for input: {}", input);
            }
        }
    }

    #[test]
    fn test_flavor_integration_in_symbol_details() {
        // Test the flavor integration logic used in Document Symbols
        use crate::flavor::registry::FlavorRegistry;

        // Create a flavor registry with Prusa flavor loaded
        let mut registry = FlavorRegistry::new();
        registry.add_embedded_prusa_flavor();
        assert!(
            registry.set_active_flavor("prusa"),
            "Should set Prusa flavor"
        );

        // Test G1 command with parameters (should get enhanced detail)
        let parsed_g1 = crate::parser::parse_line("G1 X10 Y20 F1500");
        if let crate::parser::ParsedLine::Command(command) = parsed_g1 {
            // This replicates the logic from handle_document_symbol
            let symbol_detail = registry.get_command(&command.name).map(|cmd_def| {
                let mut detail = cmd_def
                    .description_short
                    .clone()
                    .unwrap_or_else(|| "G-code command".to_string());

                // Add parameter documentation for parameters present in this command
                if !command.parameters.is_empty() {
                    if let Some(flavor_params) = &cmd_def.parameters {
                        let mut param_docs = Vec::new();

                        // Match actual parameters with flavor definitions
                        for param in &command.parameters {
                            let param_upper = param.letter.to_uppercase().to_string();
                            if let Some(flavor_param) = flavor_params
                                .iter()
                                .find(|fp| fp.name.to_uppercase() == param_upper)
                            {
                                param_docs.push(format!(
                                    "{}: {}",
                                    param.letter, flavor_param.description
                                ));
                            }
                        }

                        if !param_docs.is_empty() {
                            detail.push_str(" | ");
                            detail.push_str(&param_docs.join(", "));
                        }
                    }
                }

                detail
            });

            assert!(symbol_detail.is_some(), "G1 should have detail from flavor");
            let detail = symbol_detail.unwrap();
            assert!(
                detail.contains("Linear move"),
                "Should contain flavor description"
            );
            assert!(
                detail.contains("X: X coordinate") || detail.contains("coordinate"),
                "Should contain parameter documentation"
            );
        } else {
            panic!("Expected G1 to be parsed as command");
        }

        // Test command without flavor definition (fallback case)
        let symbol_detail_none = registry.get_command("UNKNOWN123").map(|cmd_def| {
            cmd_def
                .description_short
                .clone()
                .unwrap_or_else(|| "G-code command".to_string())
        });
        assert!(
            symbol_detail_none.is_none(),
            "Unknown command should have no detail"
        );
    }

    #[test]
    fn test_fallback_without_flavor_definition() {
        // Test that symbol detail generation works gracefully when command is not in flavor registry
        use crate::flavor::registry::FlavorRegistry;

        // Create empty flavor registry (no command definitions)
        let registry = FlavorRegistry::new();

        // Test unknown G-code command (should return None)
        let parsed_unknown = crate::parser::parse_line("G999 X10");
        if let crate::parser::ParsedLine::Command(command) = parsed_unknown {
            let symbol_detail = registry.get_command(&command.name).map(|cmd_def| {
                cmd_def
                    .description_short
                    .clone()
                    .unwrap_or_else(|| "G-code command".to_string())
            });

            assert!(
                symbol_detail.is_none(),
                "Unknown G-code command should have no detail"
            );
        } else {
            panic!("Expected G999 to be parsed as command");
        }

        // Test G1 without flavor (should return None)
        let parsed_g1 = crate::parser::parse_line("G1 Y20");
        if let crate::parser::ParsedLine::Command(command) = parsed_g1 {
            let symbol_detail = registry.get_command(&command.name).map(|cmd_def| {
                cmd_def
                    .description_short
                    .clone()
                    .unwrap_or_else(|| "G-code command".to_string())
            });

            assert!(
                symbol_detail.is_none(),
                "G1 without flavor should have no detail"
            );
        } else {
            panic!("Expected G1 to be parsed as command");
        }

        // Verify that the symbol name generation still works without flavor info
        if let crate::parser::ParsedLine::Command(command) =
            crate::parser::parse_line("G1 X10 Y20 Z0.3")
        {
            let symbol_name = if command.parameters.is_empty() {
                command.name.clone()
            } else {
                let params: Vec<String> = command
                    .parameters
                    .iter()
                    .take(3) // Limit to first 3 parameters
                    .map(|p| format!("{}{}", p.letter, p.value))
                    .collect();
                format!("{} {}", command.name, params.join(" "))
            };

            assert_eq!(
                symbol_name, "G1 X10 Y20 Z0.3",
                "Symbol name should work without flavor"
            );
        }
    }
}
