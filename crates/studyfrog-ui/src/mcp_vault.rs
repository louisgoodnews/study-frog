//! What the Model Context Protocol server shows of a vault (STORY-0118): read-only resources and a
//! search tool. Nothing here writes to the vault or to its index.

use std::fs;

use serde_json::{Value, json};
use studyfrog_agent::mcp_route::RESOURCE_NOT_FOUND;
use studyfrog_agent::mcp_types::{
    INTERNAL_ERROR, INVALID_PARAMS, McpContent, McpError, McpRegistry, McpResourceContent,
    McpResourceTemplate, McpTool, McpToolAnnotations, mcp_error, register_resource, register_tool,
    registry_new,
};
use studyfrog_core::query_eval::{Row, run};
use studyfrog_core::query_parse::parse_query;
use studyfrog_core::time::UnixMs;
use studyfrog_md::ulid::{format_ulid, parse_ulid};
use studyfrog_service::library_rows::{LibraryItem, load_library_rows};
use studyfrog_service::session::VaultSession;
use studyfrog_store::overview::load_overview;

/// What the vault tools and resources work on.
pub struct VaultCtx {
    /// The open vault.
    pub session: VaultSession,
    /// The time now, in Unix milliseconds (a parameter so tests need no clock).
    pub now_ms: fn() -> i64,
}

/// Most items a search returns.
const MAX_HITS: usize = 100;
const MS_PER_DAY: i64 = 86_400_000;

const READ_ONLY: McpToolAnnotations = McpToolAnnotations {
    title: "Search the vault",
    read_only_hint: true,
    destructive_hint: false,
    idempotent_hint: true,
    open_world_hint: false,
};

fn internal(e: impl ToString) -> McpError {
    mcp_error(INTERNAL_ERROR, e.to_string())
}

fn part<'a>(parts: &'a [(String, String)], name: &str) -> &'a str {
    parts
        .iter()
        .find(|(n, _)| n == name)
        .map_or("", |(_, v)| v.as_str())
}

fn rows_of(ctx: &VaultCtx) -> Result<Vec<LibraryItem>, McpError> {
    load_library_rows(&ctx.session, UnixMs((ctx.now_ms)())).map_err(internal)
}

fn kind_word(item: &LibraryItem) -> String {
    format!("{:?}", item.row.kind).to_lowercase()
}

fn json_content(uri: &str, value: &Value) -> McpResourceContent {
    McpResourceContent {
        uri: uri.to_owned(),
        mime_type: "application/json",
        text: serde_json::to_string_pretty(value).unwrap_or_default(),
    }
}

fn read_summary(
    ctx: &mut VaultCtx,
    uri: &str,
    _: &[(String, String)],
) -> Result<McpResourceContent, McpError> {
    let overview = load_overview(&ctx.session.conn).map_err(internal)?;
    let stacks: Vec<Value> = overview
        .stacks
        .iter()
        .map(|s| {
            json!({
                "id": format_ulid(s.uid), "title": s.title, "folder": s.folder,
                "parent": s.parent, "notes": s.notes, "cards": s.cards, "questions": s.questions,
            })
        })
        .collect();
    let name = ctx
        .session
        .root
        .file_name()
        .map(|n| n.to_string_lossy().into_owned());
    let summary =
        json!({"vault": name, "stacks": stacks, "items_outside_stacks": overview.inbox_items});
    Ok(json_content(uri, &summary))
}

fn read_stack(
    ctx: &mut VaultCtx,
    uri: &str,
    parts: &[(String, String)],
) -> Result<McpResourceContent, McpError> {
    let uid = parse_ulid(part(parts, "stack"))
        .ok_or_else(|| mcp_error(INVALID_PARAMS, "the stack id is not valid"))?;
    let overview = load_overview(&ctx.session.conn).map_err(internal)?;
    let stack = overview
        .stacks
        .iter()
        .find(|s| s.uid == uid)
        .ok_or_else(|| mcp_error(RESOURCE_NOT_FOUND, format!("no stack {}", format_ulid(uid))))?;
    let items: Vec<Value> = rows_of(ctx)?
        .iter()
        .filter(|i| i.row.stacks.contains(&stack.folder))
        .map(|i| json!({"id": format_ulid(i.uid), "title": i.row.title, "kind": kind_word(i)}))
        .collect();
    let value = json!({
        "id": format_ulid(stack.uid), "title": stack.title, "folder": stack.folder,
        "parent": stack.parent, "subjects": stack.subjects, "items": items,
    });
    Ok(json_content(uri, &value))
}

fn read_item(
    ctx: &mut VaultCtx,
    uri: &str,
    parts: &[(String, String)],
) -> Result<McpResourceContent, McpError> {
    let uid = parse_ulid(part(parts, "id"))
        .ok_or_else(|| mcp_error(INVALID_PARAMS, "the item id is not valid"))?;
    let items = rows_of(ctx)?;
    let item = items
        .iter()
        .find(|i| i.uid == uid)
        .ok_or_else(|| mcp_error(RESOURCE_NOT_FOUND, format!("no item {}", format_ulid(uid))))?;
    let (full, base) = (
        ctx.session.root.join(&item.row.path),
        ctx.session.root.clone(),
    );
    let inside =
        matches!((full.canonicalize(), base.canonicalize()), (Ok(f), Ok(b)) if f.starts_with(&b));
    if !inside {
        return Err(mcp_error(
            RESOURCE_NOT_FOUND,
            "the item's file is not inside the vault",
        ));
    }
    let text = fs::read_to_string(&full).map_err(internal)?;
    Ok(McpResourceContent {
        uri: uri.to_owned(),
        mime_type: "text/markdown",
        text,
    })
}

fn hit(item: &LibraryItem) -> Value {
    json!({
        "id": format_ulid(item.uid), "title": item.row.title, "kind": kind_word(item),
        "path": item.row.path, "tags": item.row.tags,
    })
}

fn text_result(text: String, is_error: bool) -> McpContent {
    McpContent { text, is_error }
}

/// Plain words as a text search, for a text that is not a query. A text with operators in it was
/// meant as a query, so its mistake is reported instead.
fn as_text_search(text: &str) -> Option<studyfrog_core::query::Query> {
    if text.contains(['=', '<', '>', '~', '!', '(', ')']) {
        return None;
    }
    let words = text.replace('"', " ");
    parse_query(&format!("text ~ \"{}\"", words.trim())).ok()
}

fn search(ctx: &mut VaultCtx, args: &Value) -> Result<McpContent, McpError> {
    let text = args
        .get("query")
        .and_then(Value::as_str)
        .ok_or_else(|| mcp_error(INVALID_PARAMS, "the search needs a query"))?;
    let limit = args
        .get("limit")
        .and_then(Value::as_u64)
        .map_or(20, |n| usize::try_from(n).unwrap_or(MAX_HITS))
        .clamp(1, MAX_HITS);
    let query = match parse_query(text).or_else(|e| as_text_search(text).ok_or(e)) {
        Ok(query) => query,
        Err(e) => {
            let why = format!(
                "could not read the query at column {}: {}",
                e.column, e.message
            );
            return Ok(text_result(why, true));
        }
    };
    let items = rows_of(ctx)?;
    let rows: Vec<Row> = items.iter().map(|i| i.row.clone()).collect();
    let today = (ctx.now_ms)().div_euclid(MS_PER_DAY);
    let hits: Vec<Value> = run(&query, &rows, today)
        .into_iter()
        .take(limit)
        .map(|at| hit(&items[at]))
        .collect();
    let out = json!({"count": hits.len(), "items": hits});
    Ok(text_result(
        serde_json::to_string_pretty(&out).unwrap_or_default(),
        false,
    ))
}

/// The resources and tools of a vault, all read-only.
pub fn vault_registry(version: &'static str) -> McpRegistry<VaultCtx> {
    let mut reg = registry_new("studyfrog", version);
    reg = register_resource(
        reg,
        McpResourceTemplate {
            uri_template: "studyfrog://vault/summary",
            name: "Vault summary",
            description: "The vault's name, its stacks with their item counts and the items outside every stack.",
            mime_type: "application/json",
            read: read_summary,
        },
    );
    reg = register_resource(
        reg,
        McpResourceTemplate {
            uri_template: "studyfrog://stacks/{stack}",
            name: "Stack",
            description: "A stack by its id: title, folder, subjects and the items in it.",
            mime_type: "application/json",
            read: read_stack,
        },
    );
    reg = register_resource(
        reg,
        McpResourceTemplate {
            uri_template: "studyfrog://items/{id}",
            name: "Item",
            description: "The Markdown file of an item by its id.",
            mime_type: "text/markdown",
            read: read_item,
        },
    );
    register_tool(
        reg,
        McpTool {
            name: "vault_search_items",
            description: "Find items with the StudyFrog query language, such as `type = card AND tag = exam` or `stack = French`; plain words are searched in the text. Returns id, title, kind, path and tags.",
            input_schema: r#"{"type":"object","properties":{"query":{"type":"string","description":"A StudyFrog query such as type = card AND tag = exam, or plain words"},"limit":{"type":"integer","minimum":1,"maximum":100,"default":20}},"required":["query"]}"#,
            annotations: READ_ONLY,
            call: search,
        },
    )
}
