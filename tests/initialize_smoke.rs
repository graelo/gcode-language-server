use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;

#[test]
fn initialize_smoke() {
    // Locate the built binary. Cargo sets CARGO_BIN_EXE_<name> for integration tests.
    let bin = std::env::var("CARGO_BIN_EXE_gcode-language-server").unwrap_or_else(|_| {
        // Fallback to relative path used during local runs
        "target/debug/gcode-language-server".to_string()
    });

    let mut child = Command::new(bin)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("GCODE_LS_TEST_EXIT", "1")
        .spawn()
        .expect("spawn server");

    let init = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "processId": null,
            "rootUri": null,
            "capabilities": {},
            "clientInfo": { "name": "test-client", "version": "1.0" }
        }
    });
    let body = init.to_string();
    let req = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);

    // write request
    let mut stdin = child.stdin.take().expect("child stdin");
    stdin.write_all(req.as_bytes()).expect("write init request");
    stdin.flush().expect("flush stdin");
    drop(stdin); // close stdin to signal we're done sending

    // read response with timeout
    let stdout = child.stdout.take().expect("child stdout");
    let mut reader = BufReader::new(stdout);

    let start = Instant::now();
    let timeout = Duration::from_secs(3);

    // Read Content-Length header
    let mut headers = Vec::new();
    loop {
        if start.elapsed() > timeout {
            let _ = child.kill();
            panic!("timeout waiting for response headers");
        }

        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => {
                let _ = child.kill();
                panic!("EOF while reading headers");
            }
            Ok(_) => {
                if line.trim().is_empty() {
                    break; // end of headers
                }
                headers.push(line);
            }
            Err(e) => {
                let _ = child.kill();
                panic!("error reading headers: {}", e);
            }
        }
    }

    // Parse Content-Length
    let mut content_length = None;
    for header in &headers {
        if let Some(rest) = header.strip_prefix("Content-Length:") {
            content_length = rest.trim().parse::<usize>().ok();
            break;
        }
    }

    let content_length = content_length.expect("missing Content-Length header");

    // Read exact number of body bytes
    let mut body_bytes = vec![0u8; content_length];
    std::io::Read::read_exact(&mut reader, &mut body_bytes).expect("read response body");

    let body_str = String::from_utf8(body_bytes).expect("valid utf8 response");

    // Parse and validate JSON-RPC response
    let response: Value = serde_json::from_str(&body_str)
        .unwrap_or_else(|e| panic!("invalid json response: {} - body: {}", e, body_str));

    assert_eq!(
        response.get("jsonrpc").and_then(|v| v.as_str()),
        Some("2.0"),
        "missing jsonrpc field"
    );
    assert_eq!(
        response.get("id").and_then(|v| v.as_i64()),
        Some(1),
        "wrong id field"
    );
    assert!(response.get("result").is_some(), "missing result field");

    // Ensure server process terminates cleanly
    // Give it a moment to exit naturally, then force kill if needed
    std::thread::sleep(Duration::from_millis(100));
    match child.try_wait() {
        Ok(Some(status)) => {
            assert!(
                status.success() || status.code() == Some(0),
                "server didn't exit cleanly"
            );
        }
        Ok(None) => {
            // Still running, kill it
            let _ = child.kill();
            let _ = child.wait().expect("wait after kill");
        }
        Err(e) => panic!("error checking child status: {}", e),
    }
}
