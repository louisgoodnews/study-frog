#![allow(dead_code, unused_imports, clippy::unwrap_used, clippy::expect_used)] // shared test helpers
//! Helpers for the vault MCP tests: a session over lines on a sample vault.
pub use std::collections::BTreeMap;
pub use std::io::Cursor;
pub use std::path::Path;

pub use crate::common::sample_vault;
pub use serde_json::{Value, json};
pub use studyfrog_agent::mcp_types::McpPolicy;
pub use studyfrog_core::uid::Uid;
pub use studyfrog_md::ulid::format_ulid;
pub use studyfrog_service::session::open_vault;
pub use studyfrog_ui::mcp_stdio::{McpArgs, mcp_config_snippet, parse_mcp_args, serve_lines};
pub use studyfrog_ui::mcp_vault::{VaultCtx, vault_registry};

pub fn now() -> i64 {
    1_800_000_000_000
}

pub fn ctx(path: &Path) -> VaultCtx {
    let (session, _) = open_vault(&path.display().to_string()).unwrap();
    VaultCtx {
        session,
        now_ms: now,
    }
}

/// Send these messages (one per line) and return the answers.
pub fn session(ctx: &mut VaultCtx, policy: McpPolicy, messages: &[Value]) -> Vec<Value> {
    let input: String = messages.iter().map(|m| format!("{m}\n")).collect();
    let mut out = Vec::new();
    serve_lines(
        &vault_registry("9.9.9"),
        &policy,
        ctx,
        Cursor::new(input),
        &mut out,
    )
    .unwrap();
    String::from_utf8(out)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

pub fn request(id: u32, method: &str, params: &Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params})
}

pub fn search(ctx: &mut VaultCtx, query: &str) -> Value {
    let call = request(
        1,
        "tools/call",
        &json!({"name": "vault_search_items", "arguments": {"query": query}}),
    );
    session(ctx, McpPolicy::default(), &[call]).remove(0)
}

pub fn found(answer: &Value) -> Value {
    serde_json::from_str(answer["result"]["content"][0]["text"].as_str().unwrap()).unwrap()
}

pub fn titles(answer: &Value) -> Vec<String> {
    let mut t: Vec<String> = found(answer)["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["title"].as_str().unwrap().to_owned())
        .collect();
    t.sort();
    t
}

pub fn markdown_files(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut files = BTreeMap::new();
    let mut todo = vec![root.to_path_buf()];
    while let Some(dir) = todo.pop() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                todo.push(path);
            } else if path.extension().is_some_and(|e| e == "md") {
                files.insert(
                    path.strip_prefix(root).unwrap().display().to_string(),
                    std::fs::read(&path).unwrap(),
                );
            }
        }
    }
    files
}
