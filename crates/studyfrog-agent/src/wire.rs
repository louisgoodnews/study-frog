//! The requests and answers of the protocols.

use serde_json::{Value, json};

use crate::provider::{HttpRequest, HttpResponse, Message, Protocol, Provider, Role};

/// Seconds a model gets to answer (local models can be slow).
pub const TIMEOUT_SECS: u32 = 180;
/// Most tokens a reply may have (Anthropic needs it stated).
pub const MAX_TOKENS: u32 = 4096;

fn role_word(r: Role) -> &'static str {
    match r {
        Role::User => "user",
        Role::Assistant => "assistant",
    }
}

fn gemini_role(r: Role) -> &'static str {
    match r {
        Role::User => "user",
        Role::Assistant => "model",
    }
}

fn turns(messages: &[Message]) -> Vec<Value> {
    messages
        .iter()
        .map(|m| json!({"role": role_word(m.role), "content": m.content}))
        .collect()
}

fn url(p: &Provider, path: &str) -> String {
    format!("{}{path}", p.base_url.trim_end_matches('/'))
}

/// The request that asks `p` to continue `messages` under the instructions in `system`. `key` is the
/// API key, when the provider needs one.
pub fn build_request(
    p: &Provider,
    key: Option<&str>,
    system: &str,
    messages: &[Message],
) -> Result<HttpRequest, String> {
    let json_header = ("Content-Type".to_owned(), "application/json".to_owned());
    let need_key = || {
        key.filter(|k| !k.is_empty())
            .ok_or_else(|| format!("\"{}\" needs an API key; add it first", p.name))
    };
    let (path, headers, body) = match p.protocol {
        Protocol::Ollama => {
            let mut all = vec![json!({"role": "system", "content": system})];
            all.extend(turns(messages));
            (
                "/api/chat",
                vec![json_header],
                json!({"model": p.model, "messages": all, "stream": false}),
            )
        }
        Protocol::OpenAiCompatible => {
            let mut all = vec![json!({"role": "system", "content": system})];
            all.extend(turns(messages));
            let auth = (
                "Authorization".to_owned(),
                format!(
                    "Bearer {}",
                    need_key().or_else(|e| if super::provider::is_local(p) {
                        Ok("")
                    } else {
                        Err(e)
                    })?
                ),
            );
            (
                "/v1/chat/completions",
                vec![json_header, auth],
                json!({"model": p.model, "messages": all}),
            )
        }
        Protocol::DeepSeek => {
            let mut all = vec![json!({"role": "system", "content": system})];
            all.extend(turns(messages));
            let auth = (
                "Authorization".to_owned(),
                format!("Bearer {}", need_key()?),
            );
            (
                "/chat/completions",
                vec![json_header, auth],
                json!({"model": p.model, "messages": all}),
            )
        }
        Protocol::Gemini => {
            // The key goes in a header, never in the address, so it cannot end up in a URL log.
            let headers = vec![
                json_header,
                ("x-goog-api-key".to_owned(), need_key()?.to_owned()),
            ];
            let contents: Vec<Value> = messages
                .iter()
                .map(|m| json!({"role": gemini_role(m.role), "parts": [{"text": m.content}]}))
                .collect();
            let path = format!("/v1beta/models/{}:generateContent", p.model);
            return Ok(HttpRequest {
                url: url(p, &path),
                headers,
                body: json!({
                    "system_instruction": {"parts": [{"text": system}]},
                    "contents": contents,
                    "generationConfig": {"maxOutputTokens": MAX_TOKENS},
                })
                .to_string(),
                timeout_secs: TIMEOUT_SECS,
            });
        }
        Protocol::Anthropic => {
            let headers = vec![
                json_header,
                ("x-api-key".to_owned(), need_key()?.to_owned()),
                ("anthropic-version".to_owned(), "2023-06-01".to_owned()),
            ];
            (
                "/v1/messages",
                headers,
                json!({"model": p.model, "max_tokens": MAX_TOKENS, "system": system, "messages": turns(messages)}),
            )
        }
    };
    Ok(HttpRequest {
        url: url(p, path),
        headers,
        body: body.to_string(),
        timeout_secs: TIMEOUT_SECS,
    })
}

/// A short, safe excerpt of an error body (never the request, so a key cannot leak through it).
fn excerpt(body: &str) -> String {
    let text: String = body.chars().filter(|c| !c.is_control()).take(300).collect();
    if let Ok(v) = serde_json::from_str::<Value>(body) {
        let msg = v
            .pointer("/error/message")
            .or_else(|| v.get("error"))
            .and_then(Value::as_str);
        if let Some(m) = msg {
            return m.chars().take(300).collect();
        }
    }
    text
}

/// The text of the model's answer.
pub fn parse_reply(p: &Provider, r: &HttpResponse) -> Result<String, String> {
    if !(200..300).contains(&r.status) {
        let hint = match r.status {
            401 | 403 => " (the API key was refused)",
            404 => " (is the model name right, and is the server running?)",
            429 => " (too many requests; wait a moment)",
            _ => "",
        };
        return Err(format!(
            "{} answered {}{hint}: {}",
            p.name,
            r.status,
            excerpt(&r.body)
        ));
    }
    let v: Value = serde_json::from_str(&r.body)
        .map_err(|_| format!("{} sent something that is not JSON", p.name))?;
    let text = match p.protocol {
        Protocol::Ollama => v
            .pointer("/message/content")
            .and_then(Value::as_str)
            .map(str::to_owned),
        Protocol::Gemini => return gemini_text(p, &v),
        Protocol::OpenAiCompatible | Protocol::DeepSeek => v
            .pointer("/choices/0/message/content")
            .and_then(Value::as_str)
            .map(str::to_owned),
        Protocol::Anthropic => v.get("content").and_then(Value::as_array).map(|parts| {
            parts
                .iter()
                .filter_map(|p| p.get("text").and_then(Value::as_str))
                .collect::<Vec<_>>()
                .join("")
        }),
    };
    text.filter(|t| !t.trim().is_empty())
        .ok_or_else(|| format!("{} answered without any text", p.name))
}

/// The text of a Gemini answer. Without text it says why: a safety filter or another stop reason
/// (never the content of the request).
fn gemini_text(p: &Provider, v: &Value) -> Result<String, String> {
    let parts = v
        .pointer("/candidates/0/content/parts")
        .and_then(Value::as_array);
    let text: String = parts
        .map(|parts| {
            parts
                .iter()
                .filter_map(|part| part.get("text").and_then(Value::as_str))
                .collect()
        })
        .unwrap_or_default();
    if !text.trim().is_empty() {
        return Ok(text);
    }
    let stop = v
        .pointer("/candidates/0/finishReason")
        .or_else(|| v.pointer("/promptFeedback/blockReason"))
        .and_then(Value::as_str)
        .filter(|reason| *reason != "STOP");
    Err(match stop {
        Some(reason) => format!("{} gave no answer: it stopped with {reason}", p.name),
        None => format!("{} answered without any text", p.name),
    })
}
