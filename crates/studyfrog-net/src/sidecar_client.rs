//! Server-Sent Events (SSE) streaming client and token parser (Section 4.5).
//!
//! Complies with Invariant P1 (zero inherent impls) and P6 (<= 250 LOC).

use std::io::BufRead;
use std::sync::mpsc::Sender;
use std::time::Duration;

pub use studyfrog_agent::model_provider::StreamChunk;

/// Extracts the payload from a single Server-Sent Events line.
/// Returns `Some(content)` if the line starts with `data:`, otherwise `None`.
pub fn parse_sse_line(line: &str) -> Option<String> {
    let trimmed = line.trim_end_matches(['\r', '\n']);
    let rest = trimmed.strip_prefix("data:")?;
    let content = rest.strip_prefix(' ').unwrap_or(rest);
    Some(content.to_string())
}

/// Extracts a token string from a llama-server or OpenAI-compatible JSON chunk.
pub fn extract_llama_token(json_str: &str) -> Option<String> {
    let val: serde_json::Value = serde_json::from_str(json_str).ok()?;

    // 1. llama-server native /completion format: {"content": "..."}
    if let Some(c) = val.get("content").and_then(|v| v.as_str()) {
        return Some(c.to_string());
    }

    // 2. OpenAI /v1/chat/completions format: choices[0].delta.content
    if let Some(c) = val
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|first| first.get("delta"))
        .and_then(|delta| delta.get("content"))
        .and_then(|v| v.as_str())
    {
        return Some(c.to_string());
    }

    // 3. OpenAI legacy text format: choices[0].text
    if let Some(t) = val
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|first| first.get("text"))
        .and_then(|v| v.as_str())
    {
        return Some(t.to_string());
    }

    // 4. Generic token field fallback: {"token": "..."}
    if let Some(t) = val.get("token").and_then(|v| v.as_str()) {
        return Some(t.to_string());
    }

    None
}

fn extract_thinking(val: &serde_json::Value) -> Option<String> {
    if let Some(t) = val.get("thinking").and_then(|v| v.as_str()) {
        return Some(t.to_string());
    }
    val.get("choices")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|first| first.get("delta"))
        .and_then(|delta| {
            delta
                .get("reasoning_content")
                .or_else(|| delta.get("thinking"))
        })
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn extract_error(val: &serde_json::Value) -> Option<String> {
    let err = val.get("error")?;
    if let Some(msg) = err.get("message").and_then(|m| m.as_str()) {
        return Some(msg.to_string());
    }
    if let Some(s) = err.as_str() {
        return Some(s.to_string());
    }
    Some(err.to_string())
}

/// Parses an SSE text chunk (which may contain multiple lines or events) into [`StreamChunk`]s.
pub fn parse_sse_chunk(chunk: &str) -> Vec<StreamChunk> {
    let mut chunks = Vec::new();
    let mut current_event: Option<String> = None;

    for raw_line in chunk.lines() {
        let line = raw_line.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            current_event = None;
            continue;
        }

        if let Some(ev) = line.strip_prefix("event:") {
            current_event = Some(ev.trim().to_string());
            continue;
        }

        let Some(data) = parse_sse_line(line) else {
            continue;
        };

        if data.trim() == "[DONE]" {
            chunks.push(StreamChunk::Complete(String::new()));
            continue;
        }

        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&data) {
            if let Some(err) = extract_error(&val) {
                chunks.push(StreamChunk::Error(err));
                continue;
            }

            if let Some(thinking) = extract_thinking(&val)
                && !thinking.is_empty()
            {
                chunks.push(StreamChunk::Thinking(thinking));
            }

            let is_stop = val.get("stop").and_then(|s| s.as_bool()).unwrap_or(false);
            if let Some(tok) = extract_llama_token(&data)
                && !tok.is_empty()
            {
                chunks.push(StreamChunk::Token(tok));
            }
            if is_stop {
                chunks.push(StreamChunk::Complete(String::new()));
            }
        } else if current_event.as_deref() == Some("error") || data.starts_with("error:") {
            let err_msg = data.strip_prefix("error:").unwrap_or(&data).trim();
            chunks.push(StreamChunk::Error(err_msg.to_string()));
        } else if current_event.as_deref() == Some("thinking") {
            chunks.push(StreamChunk::Thinking(data));
        } else {
            chunks.push(StreamChunk::Token(data));
        }
    }

    chunks
}

/// Reads an SSE byte stream from a [`BufRead`] line by line and dispatches [`StreamChunk`] items
/// over a channel until completion, error, or EOF.
pub fn pipe_sse_stream<R: BufRead>(
    mut reader: R,
    sender: &Sender<StreamChunk>,
) -> Result<(), String> {
    let mut line = String::new();
    while reader.read_line(&mut line).map_err(|e| e.to_string())? > 0 {
        let chunks = parse_sse_chunk(&line);
        for chunk in chunks {
            let is_complete = matches!(chunk, StreamChunk::Complete(_));
            if sender.send(chunk).is_err() {
                return Ok(());
            }
            if is_complete {
                return Ok(());
            }
        }
        line.clear();
    }
    Ok(())
}

/// Returns an iterator of [`StreamChunk`]s parsed from a [`BufRead`] stream.
pub fn sse_chunks_iter<R: BufRead>(reader: R) -> impl Iterator<Item = StreamChunk> {
    reader
        .lines()
        .map_while(Result::ok)
        .flat_map(|line| parse_sse_chunk(&line))
}

/// Executes a streaming HTTP POST request to a local sidecar or OpenAI-compatible endpoint,
/// streaming received tokens through `sender`.
pub fn stream_completion_request(
    url: &str,
    api_key: Option<&str>,
    body_json: &str,
    sender: &Sender<StreamChunk>,
) -> Result<(), String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(120)))
        .http_status_as_error(false)
        .build()
        .into();
    let mut req = agent.post(url).header("Content-Type", "application/json");
    if let Some(key) = api_key {
        req = req.header("Authorization", &format!("Bearer {key}"));
    }
    let mut resp = req
        .send(body_json)
        .map_err(|e| format!("HTTP request to {url} failed: {e}"))?;

    let status = resp.status().as_u16();
    if !(200..300).contains(&status) {
        let body = resp
            .body_mut()
            .read_to_string()
            .unwrap_or_else(|_| "Unknown HTTP error".to_string());
        let err = format!("HTTP {status}: {body}");
        let _ = sender.send(StreamChunk::Error(err.clone()));
        return Err(err);
    }

    let reader = std::io::BufReader::new(resp.body_mut().as_reader());
    pipe_sse_stream(reader, sender)
}
