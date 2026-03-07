//! MCP stdio proxy with response filtering.
//!
//! Sits between the MCP client and any MCP server, forwarding JSON-RPC
//! messages bidirectionally over stdio. Intercepts `tools/call` responses and
//! routes them through structure-aware filters to strip context bloat.
//!
//! # Protocol
//!
//! MCP stdio commonly uses JSON-RPC 2.0 framed with `Content-Length` headers.
//! Some local tooling still emits newline-delimited JSON. The proxy supports
//! both forms and preserves the framing it received for each message.

use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MessageFormat {
    ContentLength,
    NewlineDelimited,
}

#[derive(Debug, Eq, PartialEq)]
struct WireMessage {
    payload: String,
    format: MessageFormat,
}

/// Run the MCP proxy.
pub fn run_proxy(
    server_cmd: &str,
    server_args: &[String],
    no_filter: bool,
    verbose: u8,
) -> Result<()> {
    if verbose > 0 {
        eprintln!(
            "[clov-mcp] Starting proxy for: {} {:?}",
            server_cmd, server_args
        );
    }

    let mut child = Command::new(server_cmd)
        .args(server_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| {
            format!(
                "Failed to spawn MCP server: {} {:?}",
                server_cmd, server_args
            )
        })?;

    let pending_requests: Arc<Mutex<HashMap<Value, String>>> = Arc::new(Mutex::new(HashMap::new()));

    let child_stdin = child
        .stdin
        .take()
        .context("Failed to capture MCP server stdin")?;
    let child_stdout = child
        .stdout
        .take()
        .context("Failed to capture MCP server stdout")?;

    let pending_clone = Arc::clone(&pending_requests);
    let stdin_thread = thread::spawn(move || {
        let stdin = io::stdin();
        let reader = BufReader::new(stdin.lock());
        let writer = io::BufWriter::new(child_stdin);
        proxy_client_to_server(reader, writer, pending_clone, verbose)
    });

    let stdout = io::stdout();
    let reader = BufReader::new(child_stdout);
    let writer = io::BufWriter::new(stdout.lock());
    let result = proxy_server_to_client(reader, writer, &pending_requests, no_filter, verbose);

    let _ = child.wait();
    let _ = stdin_thread.join();

    result
}

fn proxy_client_to_server<R: BufRead, W: Write>(
    mut reader: R,
    mut writer: W,
    pending_requests: Arc<Mutex<HashMap<Value, String>>>,
    verbose: u8,
) -> Result<()> {
    while let Some(message) = read_mcp_message(&mut reader)? {
        if let Ok(msg) = serde_json::from_str::<Value>(&message.payload) {
            track_tool_call_request(&msg, &pending_requests, verbose);
        }

        write_mcp_message(&mut writer, &message)?;
    }

    writer.flush()?;
    Ok(())
}

fn proxy_server_to_client<R: BufRead, W: Write>(
    mut reader: R,
    mut writer: W,
    pending_requests: &Arc<Mutex<HashMap<Value, String>>>,
    no_filter: bool,
    verbose: u8,
) -> Result<()> {
    while let Some(mut message) = read_mcp_message(&mut reader)? {
        if !no_filter {
            if let Ok(msg) = serde_json::from_str::<Value>(&message.payload) {
                let filtered = filter_tool_response(msg, pending_requests, verbose);
                message.payload = serde_json::to_string(&filtered).unwrap_or(message.payload);
            }
        }

        write_mcp_message(&mut writer, &message)?;
    }

    writer.flush()?;
    Ok(())
}

fn read_mcp_message<R: BufRead>(reader: &mut R) -> Result<Option<WireMessage>> {
    let first_line = match read_line_preserve_crlf(reader)? {
        Some(line) => line,
        None => return Ok(None),
    };

    if first_line.trim().is_empty() {
        return read_mcp_message(reader);
    }

    if first_line
        .to_ascii_lowercase()
        .starts_with("content-length:")
    {
        return read_content_length_message(reader, first_line);
    }

    Ok(Some(WireMessage {
        payload: first_line.trim_end_matches(['\r', '\n']).to_string(),
        format: MessageFormat::NewlineDelimited,
    }))
}

fn read_content_length_message<R: BufRead>(
    reader: &mut R,
    first_header: String,
) -> Result<Option<WireMessage>> {
    let mut headers = vec![first_header];

    loop {
        let line = match read_line_preserve_crlf(reader)? {
            Some(line) => line,
            None => return Err(anyhow!("Unexpected EOF while reading MCP headers")),
        };

        if line == "\n" || line == "\r\n" {
            break;
        }

        headers.push(line);
    }

    let content_length = headers
        .iter()
        .find_map(|line| parse_content_length(line))
        .ok_or_else(|| anyhow!("Missing Content-Length header in MCP message"))?;

    let mut buf = vec![0u8; content_length];
    reader.read_exact(&mut buf)?;
    let payload = String::from_utf8(buf).context("MCP message body is not valid UTF-8")?;

    Ok(Some(WireMessage {
        payload,
        format: MessageFormat::ContentLength,
    }))
}

fn write_mcp_message<W: Write>(writer: &mut W, message: &WireMessage) -> Result<()> {
    match message.format {
        MessageFormat::ContentLength => {
            write!(
                writer,
                "Content-Length: {}\r\n\r\n{}",
                message.payload.len(),
                message.payload
            )?;
        }
        MessageFormat::NewlineDelimited => {
            writeln!(writer, "{}", message.payload)?;
        }
    }

    writer.flush()?;
    Ok(())
}

fn read_line_preserve_crlf<R: BufRead>(reader: &mut R) -> Result<Option<String>> {
    let mut buf = String::new();
    let bytes = reader.read_line(&mut buf)?;
    if bytes == 0 {
        return Ok(None);
    }
    Ok(Some(buf))
}

fn parse_content_length(header_line: &str) -> Option<usize> {
    let (name, value) = header_line.split_once(':')?;
    if !name.trim().eq_ignore_ascii_case("Content-Length") {
        return None;
    }
    value.trim().parse().ok()
}

fn track_tool_call_request(msg: &Value, pending: &Arc<Mutex<HashMap<Value, String>>>, verbose: u8) {
    let method = msg.get("method").and_then(|m| m.as_str());
    let id = msg.get("id");
    let tool_name = msg
        .get("params")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str());

    if let (Some("tools/call"), Some(id), Some(name)) = (method, id, tool_name) {
        if verbose > 0 {
            eprintln!("[clov-mcp] Tracking request id={} tool={}", id, name);
        }
        if let Ok(mut map) = pending.lock() {
            map.insert(id.clone(), name.to_string());
        }
    }
}

fn filter_tool_response(
    mut msg: Value,
    pending: &Arc<Mutex<HashMap<Value, String>>>,
    verbose: u8,
) -> Value {
    if !is_tool_call_response(&msg) {
        return msg;
    }

    let id = match msg.get("id") {
        Some(id) => id.clone(),
        None => return msg,
    };

    let tool_name = {
        let mut map = match pending.lock() {
            Ok(m) => m,
            Err(_) => return msg,
        };
        map.remove(&id)
    };

    let tool_name = match tool_name {
        Some(name) => name,
        None => return msg,
    };

    if verbose > 0 {
        eprintln!("[clov-mcp] Filtering response for tool: {}", tool_name);
    }

    let context = crate::universal_filter::FilterContext::default();
    let mut total_input = 0usize;
    let mut total_output = 0usize;
    let mut tracking_input = String::new();
    let mut tracking_output = String::new();

    if let Some(result) = msg.get_mut("result") {
        if let Some(content) = result.get_mut("content") {
            if let Some(items) = content.as_array_mut() {
                for item in items.iter_mut() {
                    tracking_input.push_str(&serialize_value(item));
                    tracking_input.push('\n');
                    total_input += estimate_value_size(item);
                    filter_content_item(item, &context);
                    total_output += estimate_value_size(item);
                    tracking_output.push_str(&serialize_value(item));
                    tracking_output.push('\n');
                }
            }
        }

        if let Some(structured) = result.get_mut("structuredContent") {
            tracking_input.push_str(&serialize_value(structured));
            tracking_input.push('\n');
            total_input += estimate_value_size(structured);
            crate::universal_filter::filter_json_value(structured, &context);
            total_output += estimate_value_size(structured);
            tracking_output.push_str(&serialize_value(structured));
            tracking_output.push('\n');
        }

        if let Some(data) = result.get_mut("data") {
            tracking_input.push_str(&serialize_value(data));
            tracking_input.push('\n');
            total_input += estimate_value_size(data);
            crate::universal_filter::filter_json_value(data, &context);
            total_output += estimate_value_size(data);
            tracking_output.push_str(&serialize_value(data));
            tracking_output.push('\n');
        }
    }

    if total_input > 0 {
        #[allow(deprecated)]
        crate::tracking::track(
            "mcp-call",
            &format!("clov-mcp-{}", tool_name),
            &tracking_input,
            &tracking_output,
        );
    }

    if verbose > 0 && total_input > 0 {
        let saved = total_input.saturating_sub(total_output);
        let pct = (saved as f64 / total_input as f64) * 100.0;
        eprintln!(
            "[clov-mcp] {} → {} chars ({:.0}% saved)",
            total_input, total_output, pct
        );
    }

    msg
}

fn filter_content_item(item: &mut Value, context: &crate::universal_filter::FilterContext) {
    if item.get("type").and_then(|t| t.as_str()) == Some("text") {
        if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
            item["text"] = Value::String(crate::universal_filter::filter_response(text, context));
        }
    }

    if let Some(data) = item.get_mut("data") {
        crate::universal_filter::filter_json_value(data, context);
    }

    if let Some(json) = item.get_mut("json") {
        crate::universal_filter::filter_json_value(json, context);
    }

    if let Some(structured) = item.get_mut("structuredContent") {
        crate::universal_filter::filter_json_value(structured, context);
    }
}

fn estimate_value_size(value: &Value) -> usize {
    match value {
        Value::String(text) => text.len(),
        _ => serde_json::to_string(value).map(|text| text.len()).unwrap_or(0),
    }
}

fn serialize_value(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_default()
}

fn is_tool_call_response(msg: &Value) -> bool {
    msg.get("id").is_some()
        && (msg.get("result").is_some() || msg.get("error").is_some())
        && msg.get("method").is_none()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Cursor;

    #[test]
    fn parses_newline_delimited_message() {
        let mut reader = Cursor::new(b"{\"jsonrpc\":\"2.0\"}\n".to_vec());
        let message = read_mcp_message(&mut reader).unwrap().unwrap();
        assert_eq!(message.format, MessageFormat::NewlineDelimited);
        assert_eq!(message.payload, "{\"jsonrpc\":\"2.0\"}");
    }

    #[test]
    fn parses_content_length_message() {
        let payload = "{\"jsonrpc\":\"2.0\",\"id\":1}";
        let raw = format!("Content-Length: {}\r\n\r\n{}", payload.len(), payload);
        let mut reader = Cursor::new(raw.into_bytes());
        let message = read_mcp_message(&mut reader).unwrap().unwrap();
        assert_eq!(message.format, MessageFormat::ContentLength);
        assert_eq!(message.payload, payload);
    }

    #[test]
    fn writes_content_length_message() {
        let message = WireMessage {
            payload: "{\"ok\":true}".to_string(),
            format: MessageFormat::ContentLength,
        };
        let mut out = Vec::new();
        write_mcp_message(&mut out, &message).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.starts_with("Content-Length: 11\r\n\r\n"));
        assert!(text.ends_with("{\"ok\":true}"));
    }

    #[test]
    fn filters_text_and_structured_payloads() {
        let pending = Arc::new(Mutex::new(HashMap::new()));
        pending
            .lock()
            .unwrap()
            .insert(Value::from(7), "web_search".to_string());

        let message = json!({
            "jsonrpc": "2.0",
            "id": 7,
            "result": {
                "content": [
                    {
                        "type": "text",
                        "text": "Home\nUseful body text that should remain.\nPrivacy"
                    }
                ],
                "structuredContent": {
                    "rows": [
                        {"id": 1, "name": "alpha", "extra": "x"},
                        {"id": 2, "name": "beta", "extra": "y"},
                        {"id": 3, "name": "gamma", "extra": "z"},
                        {"id": 4, "name": "delta", "extra": "q"},
                        {"id": 5, "name": "epsilon", "extra": "w"},
                        {"id": 6, "name": "zeta", "extra": "e"},
                        {"id": 7, "name": "eta", "extra": "r"},
                        {"id": 8, "name": "theta", "extra": "t"},
                        {"id": 9, "name": "iota", "extra": "y"}
                    ]
                }
            }
        });

        let filtered = filter_tool_response(message, &pending, 0);
        let text = filtered["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Useful body text"));
        assert!(!text.contains("Privacy"));

        let rows = filtered["result"]["structuredContent"]["rows"]
            .as_array()
            .unwrap();
        assert_eq!(rows.len(), 9);
        assert!(rows.last().unwrap()["_clov_summary"].is_string());
    }

    #[test]
    fn proxies_newline_messages_end_to_end() {
        let input = b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/call\",\"params\":{\"name\":\"search\"}}\n";
        let mut output = Vec::new();
        let pending = Arc::new(Mutex::new(HashMap::new()));

        proxy_client_to_server(Cursor::new(input), &mut output, Arc::clone(&pending), 0).unwrap();

        assert_eq!(String::from_utf8(output).unwrap(), String::from_utf8(input.to_vec()).unwrap());
        assert_eq!(
            pending.lock().unwrap().get(&Value::from(1)).cloned(),
            Some("search".to_string())
        );
    }
}
