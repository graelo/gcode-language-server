use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;

const SERVER_TIMEOUT: Duration = Duration::from_secs(5);
const SHUTDOWN_GRACE_PERIOD: Duration = Duration::from_millis(200);

#[test]
fn document_symbols_basic() {
    let mut server = spawn_server();

    // Initialize server
    let init_request = create_initialize_request();
    send_lsp_message(&mut server, &init_request);

    // Read initialization response and continue with full workflow
    let stdout = server
        .stdout
        .take()
        .expect("Child stdout should be available");
    let mut reader = BufReader::new(stdout);

    // Read init response
    let content_length = read_content_length_header(&mut reader);
    let body = read_message_body(&mut reader, content_length);
    let init_response: Value = serde_json::from_str(&body).expect("Valid JSON response");

    // Validate that document symbol capability is advertised
    validate_document_symbol_capability(&init_response);

    // Send initialized notification
    let initialized_notification = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "initialized",
        "params": {}
    });
    send_lsp_message(&mut server, &initialized_notification);

    // Open a test document
    let sample_gcode = "; Test G-code\nG28 ; Home all axes\nM104 S200 ; Heat extruder\nG1 X10 Y20 F3000 ; Move to position\n";
    let did_open = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": "file:///test.gcode",
                "languageId": "gcode",
                "version": 1,
                "text": sample_gcode
            }
        }
    });
    send_lsp_message(&mut server, &did_open);

    // Request document symbols
    let symbols_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "textDocument/documentSymbol",
        "params": {
            "textDocument": {
                "uri": "file:///test.gcode"
            }
        }
    });
    send_lsp_message(&mut server, &symbols_request);

    // Read symbols response (may need to skip log messages)
    let symbols_response = read_next_response_with_id(&mut reader, 2);

    validate_symbols_response(&symbols_response, sample_gcode);

    // Clean shutdown
    shutdown_server(server);
}

#[test]
fn document_symbols_empty_file() {
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

    // Open an empty document
    let did_open = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": "file:///empty.gcode",
                "languageId": "gcode",
                "version": 1,
                "text": ""
            }
        }
    });
    send_lsp_message(&mut server, &did_open);

    // Request document symbols
    let symbols_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "textDocument/documentSymbol",
        "params": {
            "textDocument": {
                "uri": "file:///empty.gcode"
            }
        }
    });
    send_lsp_message(&mut server, &symbols_request);

    // Read symbols response
    let symbols_response = read_next_response_with_id(&mut reader, 2);

    // Validate empty response
    assert_eq!(symbols_response.get("jsonrpc").unwrap(), "2.0");
    assert_eq!(symbols_response.get("id").unwrap(), 2);

    let result = symbols_response
        .get("result")
        .expect("Response should have result");
    let symbols = result.as_array().expect("Result should be an array");
    assert!(symbols.is_empty(), "Empty file should have no symbols");

    println!("✓ Empty file symbols test passed - no symbols found");

    // Clean shutdown
    shutdown_server(server);
}

// Helper functions (same as in initialize_smoke.rs)
fn spawn_server() -> std::process::Child {
    let bin_path = std::env::var("CARGO_BIN_EXE_gcode-ls")
        .unwrap_or_else(|_| "target/debug/gcode-ls".to_string());

    Command::new(bin_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("GCODE_LS_TEST_EXIT", "1")
        .spawn()
        .expect("Failed to spawn language server")
}

fn create_initialize_request() -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "processId": null,
            "rootUri": null,
            "capabilities": {
                "textDocument": {
                    "documentSymbol": { "dynamicRegistration": false }
                }
            },
            "clientInfo": { "name": "test-client", "version": "1.0" }
        }
    })
}

fn send_lsp_message(child: &mut std::process::Child, message: &Value) {
    let body = message.to_string();
    let request = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);

    let stdin = child
        .stdin
        .as_mut()
        .expect("Child stdin should be available");
    stdin
        .write_all(request.as_bytes())
        .expect("Failed to write request");
    stdin.flush().expect("Failed to flush stdin");
}

fn read_content_length_header(reader: &mut BufReader<std::process::ChildStdout>) -> usize {
    let start_time = Instant::now();
    let mut content_length = None;

    loop {
        if start_time.elapsed() > SERVER_TIMEOUT {
            panic!("Timeout waiting for response headers");
        }

        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => panic!("Unexpected EOF while reading headers"),
            Ok(_) => {
                if line.trim().is_empty() {
                    // End of headers - we've consumed the empty line
                    break;
                }

                if let Some(length_str) = line.strip_prefix("Content-Length:") {
                    content_length = Some(
                        length_str
                            .trim()
                            .parse::<usize>()
                            .expect("Invalid Content-Length header"),
                    );
                }
            }
            Err(e) => panic!("Error reading headers: {}", e),
        }
    }

    content_length.expect("Missing Content-Length header")
}

fn read_message_body(
    reader: &mut BufReader<std::process::ChildStdout>,
    content_length: usize,
) -> String {
    let mut body_bytes = vec![0u8; content_length];
    std::io::Read::read_exact(reader, &mut body_bytes).expect("Failed to read response body");

    String::from_utf8(body_bytes).expect("Response body should be valid UTF-8")
}

fn read_next_response_with_id(
    reader: &mut BufReader<std::process::ChildStdout>,
    expected_id: u64,
) -> Value {
    // Keep reading responses until we find one with the expected id
    loop {
        let content_length = read_content_length_header(reader);
        let body = read_message_body(reader, content_length);
        let response: Value = serde_json::from_str(&body).expect("Valid JSON response");

        // Check if this is the response we're looking for
        if let Some(id) = response.get("id") {
            if id.as_u64() == Some(expected_id) {
                return response;
            }
        }

        // Otherwise, this might be a notification/log message, skip it
    }
}

fn validate_document_symbol_capability(response: &Value) {
    // Validate that document symbol capability is advertised
    let capabilities = response
        .get("result")
        .and_then(|r| r.get("capabilities"))
        .expect("Response should have server capabilities");

    let document_symbol_provider = capabilities.get("documentSymbolProvider");
    assert!(
        document_symbol_provider.is_some() && !document_symbol_provider.unwrap().is_null(),
        "Server should advertise document symbol provider capability"
    );
}

fn validate_symbols_response(response: &Value, _original_text: &str) {
    assert_eq!(response.get("jsonrpc").unwrap(), "2.0");
    assert_eq!(response.get("id").unwrap(), 2);

    let result = response.get("result").expect("Response should have result");

    // Should be an array of symbols
    let symbols = result.as_array().expect("Result should be an array");

    // Should have at least 3 symbols (G28, M104, G1) from our test file
    assert!(
        symbols.len() >= 3,
        "Should have at least 3 symbols, got {}",
        symbols.len()
    );

    // Check first symbol (G28)
    let first_symbol = &symbols[0];
    let name = first_symbol.get("name").unwrap().as_str().unwrap();
    assert!(
        name.contains("G28"),
        "First symbol should be G28, got {}",
        name
    );

    // Check that symbols have required fields
    assert!(
        first_symbol.get("kind").is_some(),
        "Symbol should have kind"
    );
    assert!(
        first_symbol.get("range").is_some(),
        "Symbol should have range"
    );
    assert!(
        first_symbol.get("selectionRange").is_some(),
        "Symbol should have selectionRange"
    );

    println!(
        "✓ Document symbols test passed with {} symbols",
        symbols.len()
    );
    for symbol in symbols {
        let name = symbol.get("name").unwrap().as_str().unwrap();
        let kind = symbol.get("kind").unwrap().as_u64().unwrap();
        let detail = symbol
            .get("detail")
            .and_then(|d| d.as_str())
            .unwrap_or("No detail");
        println!("  Symbol: {} (kind: {}) - {}", name, kind, detail);
    }
}

fn shutdown_server(mut child: std::process::Child) {
    // Close stdin to signal we're done
    drop(child.stdin.take());

    // Give the server a moment to exit gracefully
    std::thread::sleep(SHUTDOWN_GRACE_PERIOD);

    match child.try_wait() {
        Ok(Some(status)) => {
            if !status.success() {
                eprintln!("Server exited with non-zero status: {:?}", status);
            }
        }
        Ok(None) => {
            // Still running, force termination
            eprintln!("Server didn't exit gracefully, forcing termination");
            let _ = child.kill();
            let _ = child.wait();
        }
        Err(e) => panic!("Error checking server status: {}", e),
    }
}
