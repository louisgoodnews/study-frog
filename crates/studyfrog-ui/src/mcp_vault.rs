//! What the Model Context Protocol server shows of a vault (STORY-0118, STORY-0119): the context the
//! tools and resources work on, the helpers they share, and the registry that lists them all. The
//! read-only parts never write to the vault or its index; the authoring tools stage their changes
//! unless the server was started with permission to write.

use serde_json::Value;
use studyfrog_agent::mcp_types::{
    INTERNAL_ERROR, INVALID_PARAMS, McpContent, McpError, McpPolicy, McpRegistry,
    McpResourceContent, McpToolAnnotations, mcp_error, registry_new,
};
use studyfrog_core::time::UnixMs;
use studyfrog_service::library_rows::{LibraryItem, load_library_rows};
use studyfrog_service::session::VaultSession;

/// What the vault tools and resources work on.
pub struct VaultCtx {
    /// The open vault.
    pub session: VaultSession,
    /// The time now, in Unix milliseconds (a parameter so tests need no clock).
    pub now_ms: fn() -> i64,
    /// What the server lets through; the authoring tools write directly only with `allow_write`.
    pub policy: McpPolicy,
}

pub(crate) const MS_PER_DAY: i64 = 86_400_000;

/// The annotations of a tool that only reads.
pub(crate) const fn read_only(title: &'static str) -> McpToolAnnotations {
    McpToolAnnotations {
        title,
        read_only_hint: true,
        destructive_hint: false,
        idempotent_hint: true,
        open_world_hint: false,
    }
}

/// An unexpected failure, as a JSON-RPC error.
pub(crate) fn internal(e: impl ToString) -> McpError {
    mcp_error(INTERNAL_ERROR, e.to_string())
}

/// Wrong arguments, as a JSON-RPC error.
pub(crate) fn invalid(message: impl Into<String>) -> McpError {
    mcp_error(INVALID_PARAMS, message)
}

/// The value of the `{name}` part of an address.
pub(crate) fn part<'a>(parts: &'a [(String, String)], name: &str) -> &'a str {
    parts
        .iter()
        .find(|(n, _)| n == name)
        .map_or("", |(_, v)| v.as_str())
}

/// The now of the context.
pub(crate) fn now_of(ctx: &VaultCtx) -> UnixMs {
    UnixMs((ctx.now_ms)())
}

/// Every item of the vault as a row of the library.
pub(crate) fn rows_of(ctx: &VaultCtx) -> Result<Vec<LibraryItem>, McpError> {
    load_library_rows(&ctx.session, now_of(ctx)).map_err(internal)
}

/// `note`, `card`, `question`, ...
pub(crate) fn kind_word(item: &LibraryItem) -> String {
    format!("{:?}", item.row.kind).to_lowercase()
}

/// A JSON resource.
pub(crate) fn json_content(uri: &str, value: &Value) -> McpResourceContent {
    McpResourceContent {
        uri: uri.to_owned(),
        mime_type: "application/json",
        text: serde_json::to_string_pretty(value).unwrap_or_default(),
        more: Vec::new(),
    }
}

/// A tool's answer as JSON text.
pub(crate) fn json_result(value: &Value) -> McpContent {
    McpContent {
        text: serde_json::to_string_pretty(value).unwrap_or_default(),
        is_error: false,
    }
}

/// A tool's answer the model should read as a failure it can react to.
pub(crate) fn tool_error(text: impl Into<String>) -> McpContent {
    McpContent {
        text: text.into(),
        is_error: true,
    }
}

/// A text argument, if given and not blank.
pub(crate) fn arg_str<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

/// A whole-number argument clamped to `lo..=hi`, else `default`.
pub(crate) fn arg_count(
    args: &Value,
    key: &str,
    default: usize,
    (lo, hi): (usize, usize),
) -> usize {
    args.get(key)
        .and_then(Value::as_u64)
        .map_or(default, |n| usize::try_from(n).unwrap_or(hi))
        .clamp(lo, hi)
}

/// Every resource, tool and prompt of a vault: the read-only ones, the staged authoring tools, the
/// gated structural tools and the standard prompts.
pub fn vault_registry(version: &'static str) -> McpRegistry<VaultCtx> {
    let mut reg = registry_new("studyfrog", version);
    reg = crate::mcp_vault_read::register(reg);
    reg = crate::mcp_search::register(reg);
    reg = crate::mcp_reports::register(reg);
    reg = crate::mcp_diary_pdf::register(reg);
    reg = crate::mcp_write::register(reg);
    reg = crate::mcp_structural::register(reg);
    crate::mcp_prompts::register(reg)
}
