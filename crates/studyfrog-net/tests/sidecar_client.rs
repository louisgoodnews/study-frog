//! Integration tests for sidecar client SSE streaming parser.

use std::io::Cursor;
use std::sync::mpsc::channel;

use studyfrog_net::sidecar_client::{
    StreamChunk, extract_llama_token, parse_sse_chunk, parse_sse_line, pipe_sse_stream,
    sse_chunks_iter,
};

#[test]
fn test_parse_sse_line() {
    assert_eq!(
        parse_sse_line("data: hello world\n"),
        Some("hello world".to_string())
    );
    assert_eq!(parse_sse_line("data:hello\r\n"), Some("hello".to_string()));
    assert_eq!(parse_sse_line("data: [DONE]"), Some("[DONE]".to_string()));
    assert_eq!(parse_sse_line("event: message"), None);
    assert_eq!(parse_sse_line(": comment"), None);
    assert_eq!(parse_sse_line(""), None);
}

#[test]
fn test_extract_llama_token() {
    // 1. llama-server native format
    let native = r#"{"content": "Hello", "stop": false}"#;
    assert_eq!(extract_llama_token(native), Some("Hello".to_string()));

    // 2. OpenAI chat completion format
    let openai_chat = r#"{"choices": [{"delta": {"content": "world"}}]}"#;
    assert_eq!(extract_llama_token(openai_chat), Some("world".to_string()));

    // 3. OpenAI text format
    let openai_text = r#"{"choices": [{"text": "!"}]}"#;
    assert_eq!(extract_llama_token(openai_text), Some("!".to_string()));

    // 4. Token field fallback
    let fallback = r#"{"token": "tok"}"#;
    assert_eq!(extract_llama_token(fallback), Some("tok".to_string()));

    // 5. Invalid / non-matching
    assert_eq!(extract_llama_token("not json"), None);
    assert_eq!(extract_llama_token(r#"{"other": 123}"#), None);
}

#[test]
fn test_parse_sse_chunk_streaming_tokens() {
    let raw = "data: {\"content\": \"The\"}\n\ndata: {\"content\": \" answer\"}\n\ndata: [DONE]\n";
    let chunks = parse_sse_chunk(raw);
    assert_eq!(
        chunks,
        vec![
            StreamChunk::Token("The".to_string()),
            StreamChunk::Token(" answer".to_string()),
            StreamChunk::Complete(String::new()),
        ]
    );
}

#[test]
fn test_parse_sse_chunk_thinking_and_stop() {
    let raw = concat!(
        "data: {\"thinking\": \"Analyzing vault...\"}\n\n",
        "data: {\"content\": \"42\", \"stop\": true}\n"
    );
    let chunks = parse_sse_chunk(raw);
    assert_eq!(
        chunks,
        vec![
            StreamChunk::Thinking("Analyzing vault...".to_string()),
            StreamChunk::Token("42".to_string()),
            StreamChunk::Complete(String::new()),
        ]
    );
}

#[test]
fn test_parse_sse_chunk_errors() {
    // JSON error
    let raw_json_err = "data: {\"error\": {\"message\": \"Context length exceeded\"}}\n";
    let chunks = parse_sse_chunk(raw_json_err);
    assert_eq!(
        chunks,
        vec![StreamChunk::Error("Context length exceeded".to_string())]
    );

    // Event error
    let raw_event_err = "event: error\ndata: Connection refused\n";
    let chunks = parse_sse_chunk(raw_event_err);
    assert_eq!(
        chunks,
        vec![StreamChunk::Error("Connection refused".to_string())]
    );
}

#[test]
fn test_pipe_sse_stream_channel() {
    let data = concat!(
        "data: {\"content\": \"Hello\"}\n\n",
        "data: {\"content\": \" world\"}\n\n",
        "data: [DONE]\n"
    );
    let cursor = Cursor::new(data);
    let (tx, rx) = channel();

    let res = pipe_sse_stream(cursor, &tx);
    assert!(res.is_ok());

    let received: Vec<StreamChunk> = rx.try_iter().collect();
    assert_eq!(
        received,
        vec![
            StreamChunk::Token("Hello".to_string()),
            StreamChunk::Token(" world".to_string()),
            StreamChunk::Complete(String::new()),
        ]
    );
}

#[test]
fn test_sse_chunks_iter() {
    let data = concat!(
        "data: {\"content\": \"Alpha\"}\n",
        "data: {\"content\": \" Beta\"}\n",
        "data: [DONE]\n"
    );
    let cursor = Cursor::new(data);
    let chunks: Vec<StreamChunk> = sse_chunks_iter(cursor).collect();

    assert_eq!(
        chunks,
        vec![
            StreamChunk::Token("Alpha".to_string()),
            StreamChunk::Token(" Beta".to_string()),
            StreamChunk::Complete(String::new()),
        ]
    );
}
