use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Command, Stdio};

use serde_json::Value;

#[test]
fn test_enhanced_symbols_with_prusa_file() {
    let mut server = spawn_server();

    // Initialize server
    let init_request = create_initialize_request();
    send_lsp_message(&mut server, &init_request);

    // Read initialization response
    let stdout = server
        .stdout
        .take()
        .expect("Child stdout should be available");
    let mut reader = BufReader::new(stdout);

    let content_length = read_content_length_header(&mut reader);
    let body = read_message_body(&mut reader, content_length);
    let _init_response: Value = serde_json::from_str(&body).expect("Valid JSON response");

    // Send initialized notification
    let initialized_notification = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "initialized",
        "params": {}
    });
    send_lsp_message(&mut server, &initialized_notification);

    // Load the actual Prusa sample file
    let prusa_gcode = std::fs::read_to_string("tests/fixtures/sample_prusa.gcode")
        .expect("Should be able to read sample Prusa file");

    // Open the Prusa document
    let did_open = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": "file:///sample_prusa.gcode",
                "languageId": "gcode",
                "version": 1,
                "text": prusa_gcode
            }
        }
    });
    send_lsp_message(&mut server, &did_open);

    // Request document symbols
    let symbols_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "textDocument/documentSymbol",
        "params": {
            "textDocument": {
                "uri": "file:///sample_prusa.gcode"
            }
        }
    });
    send_lsp_message(&mut server, &symbols_request);

    // Read symbols response
    let symbols_response = read_next_response_with_id(&mut reader, 3);

    validate_prusa_symbols_response(&symbols_response);

    // Clean shutdown
    shutdown_server(server);
}

fn validate_prusa_symbols_response(response: &Value) {
    assert_eq!(response.get("jsonrpc").unwrap(), "2.0");
    assert_eq!(response.get("id").unwrap(), 3);

    let result = response.get("result").expect("Response should have result");
    let symbols = result.as_array().expect("Result should be an array");

    println!(
        "✓ Enhanced Prusa symbols test passed with {} symbols",
        symbols.len()
    );
    println!("Enhanced Document Symbols with Flavor Integration:");
    println!("================================================");

    for (i, symbol) in symbols.iter().enumerate() {
        let name = symbol.get("name").unwrap().as_str().unwrap();
        let kind = symbol.get("kind").unwrap().as_u64().unwrap();
        let detail = symbol
            .get("detail")
            .and_then(|d| d.as_str())
            .unwrap_or("No flavor definition found");

        println!("{}. Symbol: {}", i + 1, name);
        println!("   Kind: {} | Detail: {}", kind, detail);
        println!();
    }

    // Should have many symbols from the real file
    assert!(
        symbols.len() >= 10,
        "Should have at least 10 symbols from the Prusa file, got {}",
        symbols.len()
    );

    // Look for specific commands that should have enhanced details
    let symbol_names: Vec<String> = symbols
        .iter()
        .map(|s| s.get("name").unwrap().as_str().unwrap().to_string())
        .collect();

    // Should find G28 (home)
    assert!(
        symbol_names.iter().any(|name| name.contains("G28")),
        "Should find G28 command"
    );

    // Should find G92 (set position)
    assert!(
        symbol_names.iter().any(|name| name.contains("G92")),
        "Should find G92 command"
    );

    // Should find M250 (Prusa-specific LCD contrast)
    assert!(
        symbol_names.iter().any(|name| name.contains("M250")),
        "Should find M250 Prusa-specific command"
    );
}

// Helper functions (shared with other tests)
fn spawn_server() -> std::process::Child {
    Command::new("cargo")
        .args(["run", "--bin", "gcode-ls"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start language server")
}

fn create_initialize_request() -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "processId": null,
            "rootUri": null,
            "capabilities": {
                "textDocument": {
                    "documentSymbol": {
                        "dynamicRegistration": true,
                        "symbolKind": {
                            "valueSet": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26]
                        },
                        "hierarchicalDocumentSymbolSupport": true
                    }
                }
            }
        }
    })
}

fn send_lsp_message(server: &mut std::process::Child, message: &serde_json::Value) {
    let content = message.to_string();
    let full_message = format!("Content-Length: {}\r\n\r\n{}", content.len(), content);

    if let Some(stdin) = server.stdin.as_mut() {
        stdin
            .write_all(full_message.as_bytes())
            .expect("Failed to write to server");
        stdin.flush().expect("Failed to flush server input");
    }
}

fn read_content_length_header(reader: &mut BufReader<std::process::ChildStdout>) -> usize {
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .expect("Failed to read content-length header");

    let content_length_str = line
        .trim()
        .strip_prefix("Content-Length: ")
        .expect("Expected Content-Length header");
    let content_length: usize = content_length_str
        .parse()
        .expect("Content-Length should be a number");

    // Read the empty line after headers
    let mut empty_line = String::new();
    reader
        .read_line(&mut empty_line)
        .expect("Failed to read empty line");

    content_length
}

fn read_message_body(
    reader: &mut BufReader<std::process::ChildStdout>,
    content_length: usize,
) -> String {
    let mut buffer = vec![0u8; content_length];
    reader
        .read_exact(&mut buffer)
        .expect("Failed to read message body");
    String::from_utf8(buffer).expect("Message body should be valid UTF-8")
}

fn read_next_response_with_id(
    reader: &mut BufReader<std::process::ChildStdout>,
    expected_id: u64,
) -> serde_json::Value {
    loop {
        let content_length = read_content_length_header(reader);
        let body = read_message_body(reader, content_length);
        let message: serde_json::Value = serde_json::from_str(&body).expect("Valid JSON");

        // Skip log messages and notifications, return only responses with matching ID
        if let Some(id) = message.get("id") {
            if id.as_u64() == Some(expected_id) {
                return message;
            }
        }
        // Continue reading if this was a log message or different response
    }
}

fn shutdown_server(mut child: std::process::Child) {
    // Close stdin to signal we're done
    drop(child.stdin.take());

    // Wait for server to exit
    match child.wait() {
        Ok(status) => {
            if !status.success() {
                eprintln!("Server exited with non-zero status: {:?}", status);
            }
        }
        Err(e) => eprintln!("Error waiting for server: {}", e),
    }
}
