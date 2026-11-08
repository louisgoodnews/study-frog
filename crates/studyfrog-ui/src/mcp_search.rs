//! The read-only finding tools of the MCP server (STORY-0118): `vault_search_items` with the query
//! language and `vault_get_due_queue` with the queue a spaced run would rehearse now.

use serde_json::{Value, json};
use studyfrog_agent::mcp_types::{McpContent, McpError, McpRegistry, McpTool, register_tool};
use studyfrog_core::plan::plan_spaced;
use studyfrog_core::query_eval::{Row, run};
use studyfrog_core::query_parse::parse_query;
use studyfrog_core::run::RunMode;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_rehearsal::queue::candidate::budget_unlimited;
use studyfrog_service::library_rows::LibraryItem;
use studyfrog_service::queue::{NewRun, new_run};
use studyfrog_service::session::current_settings;
use studyfrog_service::timezone::local_day_window;
use studyfrog_store::overview::load_overview;

use crate::mcp_vault::{
    MS_PER_DAY, VaultCtx, arg_count, arg_str, internal, invalid, json_result, kind_word, now_of,
    read_only, rows_of, tool_error,
};
use crate::mcp_vault_read::find_stack;

/// Most items a search or a queue returns.
const MAX_HITS: usize = 100;

fn hit(item: &LibraryItem) -> Value {
    json!({
        "id": format_ulid(item.uid), "key": item.row.key, "title": item.row.title,
        "kind": kind_word(item), "path": item.row.path, "tags": item.row.tags,
        "familiarity": item.row.familiarity, "due_day": item.row.due_day,
    })
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
    let text = arg_str(args, "query").ok_or_else(|| invalid("the search needs a query"))?;
    let limit = arg_count(args, "limit", 20, (1, MAX_HITS));
    let query = match parse_query(text).or_else(|e| as_text_search(text).ok_or(e)) {
        Ok(query) => query,
        Err(e) => {
            return Ok(tool_error(format!(
                "could not read the query at column {}: {}",
                e.column, e.message
            )));
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
    Ok(json_result(&json!({"count": hits.len(), "items": hits})))
}

/// The stacks a queue covers: the one named, else every stack.
fn stacks_of(ctx: &VaultCtx, args: &Value) -> Result<Result<Vec<Uid>, String>, McpError> {
    let Some(wanted) = arg_str(args, "stack") else {
        return Ok(Ok(Vec::new()));
    };
    let overview = load_overview(&ctx.session.conn).map_err(internal)?;
    Ok(find_stack(&overview.stacks, wanted)
        .map(|s| vec![s.uid])
        .ok_or_else(|| format!("there is no stack {wanted}")))
}

fn due_queue(ctx: &mut VaultCtx, args: &Value) -> Result<McpContent, McpError> {
    let stacks = match stacks_of(ctx, args)? {
        Ok(s) => s,
        Err(why) => return Ok(tool_error(why)),
    };
    let limit = arg_count(args, "limit", 50, (1, MAX_HITS));
    let now = now_of(ctx);
    let mut plan = plan_spaced();
    plan.filter.stacks = stacks;
    let req = NewRun {
        plan,
        mode: RunMode::Flip,
        uid: Uid(0),
        now,
        day: local_day_window(now),
        global_budget: budget_unlimited(),
    };
    let queue = new_run(&ctx.session.conn, req, &current_settings(&ctx.session))
        .map_err(internal)?
        .queue;
    let items = rows_of(ctx)?;
    let listed: Vec<Value> = queue
        .iter()
        .take(limit)
        .filter_map(|uid| items.iter().find(|i| i.uid == *uid))
        .map(hit)
        .collect();
    Ok(json_result(&json!({
        "queued": queue.len(), "shown": listed.len(), "items": listed,
        "note": "The order a spaced run would rehearse now, within each stack's daily limits. Nothing was started or changed.",
    })))
}

/// Register the finding tools.
pub fn register(reg: McpRegistry<VaultCtx>) -> McpRegistry<VaultCtx> {
    let reg = register_tool(
        reg,
        McpTool {
            name: "vault_search_items",
            description: "Find items with the StudyFrog query language, such as `type = card AND tag = exam` or `stack = French`; plain words are searched in the text. Returns id, key, title, kind, path, tags, familiarity and due day.",
            input_schema: r#"{"type":"object","properties":{"query":{"type":"string","description":"A StudyFrog query such as type = card AND tag = exam, or plain words"},"limit":{"type":"integer","minimum":1,"maximum":100,"default":20}},"required":["query"]}"#,
            annotations: read_only("Search the vault"),
            staged: false,
            call: search,
        },
    );
    register_tool(
        reg,
        McpTool {
            name: "vault_get_due_queue",
            description: "The prioritized rehearsal queue a spaced run would show now, for one stack (id, folder or title) or the whole vault, within the daily limits. Starts nothing.",
            input_schema: r#"{"type":"object","properties":{"stack":{"type":"string","description":"Stack id, folder or title; omit for the whole vault"},"limit":{"type":"integer","minimum":1,"maximum":100,"default":50}}}"#,
            annotations: read_only("Due queue"),
            staged: false,
            call: due_queue,
        },
    )
}
