use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;

const SERVER_TIMEOUT: Duration = Duration::from_secs(5);
const SHUTDOWN_GRACE_PERIOD: Duration = Duration::from_millis(200);

#[test]
fn initialize_smoke() {
    let mut server = spawn_server();

    // Send initialize request
    let init_request = create_initialize_request();
    send_lsp_message(&mut server, &init_request);

    // Read and validate response
    let response = read_lsp_response(&mut server);
    validate_initialize_response(&response);

    // Clean shutdown
    shutdown_server(server);
}

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
                    "hover": { "dynamicRegistration": false },
                    "completion": { "dynamicRegistration": false }
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

fn read_lsp_response(child: &mut std::process::Child) -> Value {
    let stdout = child
        .stdout
        .take()
        .expect("Child stdout should be available");
    let mut reader = BufReader::new(stdout);

    let content_length = read_content_length_header(&mut reader);
    let body = read_message_body(&mut reader, content_length);

    serde_json::from_str(&body)
        .unwrap_or_else(|e| panic!("Invalid JSON response: {}\nBody: {}", e, body))
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

fn validate_initialize_response(response: &Value) {
    // Validate JSON-RPC structure
    assert_eq!(
        response.get("jsonrpc").and_then(|v| v.as_str()),
        Some("2.0"),
        "Response should have jsonrpc: '2.0'"
    );

    assert_eq!(
        response.get("id").and_then(|v| v.as_i64()),
        Some(1),
        "Response should have matching request id"
    );

    // Validate LSP initialize response structure
    let result = response
        .get("result")
        .expect("Response should contain 'result' field");

    let capabilities = result
        .get("capabilities")
        .expect("Result should contain server capabilities");

    // Basic capability checks - adjust based on what your server actually supports
    assert!(capabilities.is_object(), "Capabilities should be an object");

    // Optional: Check for specific capabilities your server provides
    // assert!(capabilities.get("hoverProvider").is_some(), "Should support hover");
    // assert!(capabilities.get("completionProvider").is_some(), "Should support completion");
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
