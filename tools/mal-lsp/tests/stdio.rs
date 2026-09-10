use std::io::{BufRead, Cursor, Read, Write};
use std::process::{Command, Stdio};

use serde_json::{Value, json};

fn frame(message: &Value) -> Vec<u8> {
    let body = serde_json::to_vec(message).unwrap();
    let mut framed = format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes();
    framed.extend(body);
    framed
}

#[test]
fn serves_initialize_and_shutdown_over_stdio() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_mal-lsp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let input = [
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}),
        json!({"jsonrpc": "2.0", "method": "initialized", "params": {}}),
        json!({"jsonrpc": "2.0", "id": 2, "method": "shutdown"}),
        json!({"jsonrpc": "2.0", "method": "exit"}),
    ];
    {
        let stdin = child.stdin.as_mut().unwrap();
        for message in &input {
            stdin.write_all(&frame(message)).unwrap();
        }
    }

    let output = child.wait_with_output().unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"id\":1"));
    assert!(stdout.contains("\"id\":2"));
    assert_eq!(stdout.matches("Content-Length:").count(), 2);
}

#[test]
fn invalid_source_hover_is_an_empty_result_not_a_protocol_error() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_mal-lsp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let input = [
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}),
        json!({
            "jsonrpc": "2.0", "method": "textDocument/didOpen",
            "params": {"textDocument": {
                "uri": "file:///invalid.mal", "languageId": "mal", "version": 1,
                "text": "good :: Int32 := 1;\nbad :: Int32 := ;\n"
            }}
        }),
        json!({
            "jsonrpc": "2.0", "id": 2, "method": "textDocument/hover",
            "params": {
                "textDocument": {"uri": "file:///invalid.mal"},
                "position": {"line": 0, "character": 1}
            }
        }),
        json!({"jsonrpc": "2.0", "id": 3, "method": "shutdown"}),
        json!({"jsonrpc": "2.0", "method": "exit"}),
    ];
    {
        let stdin = child.stdin.as_mut().unwrap();
        for message in &input {
            stdin.write_all(&frame(message)).unwrap();
        }
    }

    let output = child.wait_with_output().unwrap();
    let messages = decode_frames(&output.stdout);

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        messages
            .iter()
            .filter(|message| message["method"] == "textDocument/publishDiagnostics")
            .count(),
        1
    );
    let hover = messages.iter().find(|message| message["id"] == 2).unwrap();
    assert_eq!(hover["result"], Value::Null);
    assert!(hover.get("error").is_none());
}

fn decode_frames(bytes: &[u8]) -> Vec<Value> {
    let mut reader = Cursor::new(bytes);
    let mut messages = Vec::new();
    while reader.position() < bytes.len() as u64 {
        let mut content_length = None;
        loop {
            let mut header = String::new();
            reader.read_line(&mut header).unwrap();
            if header == "\r\n" {
                break;
            }
            if let Some(value) = header.strip_prefix("Content-Length:") {
                content_length = Some(value.trim().parse::<usize>().unwrap());
            }
        }
        let mut body = vec![0; content_length.unwrap()];
        reader.read_exact(&mut body).unwrap();
        messages.push(serde_json::from_slice(&body).unwrap());
    }
    messages
}
