//! The vault resources of the MCP server (STORY-0118): the summary, a stack and an item with its
//! schedule. Read-only.

use std::collections::BTreeMap;
use std::fs;

use serde_json::{Value, json};
use studyfrog_agent::mcp_route::RESOURCE_NOT_FOUND;
use studyfrog_agent::mcp_types::{
    McpError, McpRegistry, McpResourceContent, McpResourceTemplate, mcp_error, register_resource,
};
use studyfrog_md::ulid::{format_ulid, parse_ulid};
use studyfrog_service::home::home_summary;
use studyfrog_service::item_schedule::{ItemSchedule, ScheduleState, item_schedules};
use studyfrog_service::library_rows::LibraryItem;
use studyfrog_service::timezone::{local_day_window, local_offset_minutes};
use studyfrog_store::overview::{StackRow, load_overview};

use crate::mcp_vault::{
    VaultCtx, internal, invalid, json_content, kind_word, now_of, part, rows_of,
};

fn read_summary(
    ctx: &mut VaultCtx,
    uri: &str,
    _: &[(String, String)],
) -> Result<McpResourceContent, McpError> {
    let overview = load_overview(&ctx.session.conn).map_err(internal)?;
    let now = now_of(ctx);
    let home = home_summary(
        &ctx.session,
        now,
        local_offset_minutes(now),
        local_day_window(now),
    )
    .map_err(internal)?;
    let stacks: Vec<Value> = overview
        .stacks
        .iter()
        .map(|s| {
            let due = home.stacks.iter().find(|d| d.uid == s.uid);
            json!({
                "id": format_ulid(s.uid), "title": s.title, "folder": s.folder,
                "parent": s.parent, "notes": s.notes, "cards": s.cards, "questions": s.questions,
                "due": due.map_or(0, |d| d.due), "new": due.map_or(0, |d| d.new),
            })
        })
        .collect();
    let mut by_kind: BTreeMap<String, usize> = BTreeMap::new();
    for item in rows_of(ctx)? {
        *by_kind.entry(kind_word(&item)).or_default() += 1;
    }
    let name = ctx
        .session
        .root
        .file_name()
        .map(|n| n.to_string_lossy().into_owned());
    let summary = json!({
        "vault": name, "stacks": stacks, "items_outside_stacks": overview.inbox_items,
        "items_by_kind": by_kind, "due_now": home.due_now, "new_items": home.new_items,
        "reviews_today": home.reviews_today, "streak_days": home.streak_days,
    });
    Ok(json_content(uri, &summary))
}

/// `%2F` in an address part stands for `/`, so a nested folder fits in one part.
fn decoded(text: &str) -> String {
    text.replace("%2F", "/")
        .replace("%2f", "/")
        .replace("%20", " ")
}

/// The stack named by id, folder or (ignoring case) title.
pub(crate) fn find_stack<'a>(stacks: &'a [StackRow], wanted: &str) -> Option<&'a StackRow> {
    let wanted = decoded(wanted);
    let uid = parse_ulid(&wanted);
    stacks
        .iter()
        .find(|s| Some(s.uid) == uid || s.folder == wanted)
        .or_else(|| {
            stacks
                .iter()
                .find(|s| s.title.to_lowercase() == wanted.to_lowercase())
        })
}

fn read_stack(
    ctx: &mut VaultCtx,
    uri: &str,
    parts: &[(String, String)],
) -> Result<McpResourceContent, McpError> {
    let overview = load_overview(&ctx.session.conn).map_err(internal)?;
    let wanted = part(parts, "stack");
    let stack = find_stack(&overview.stacks, wanted)
        .ok_or_else(|| mcp_error(RESOURCE_NOT_FOUND, format!("no stack {wanted}")))?;
    let fam = studyfrog_service::fam_report::familiarity_report(&ctx.session.conn, now_of(ctx))
        .map_err(internal)?;
    let familiarity = fam
        .stacks
        .iter()
        .find(|g| g.folder == stack.folder)
        .and_then(|g| g.percent);
    let items: Vec<Value> = rows_of(ctx)?
        .iter()
        .filter(|i| i.row.stacks.contains(&stack.folder))
        .map(|i| json!({"id": format_ulid(i.uid), "key": i.row.key, "title": i.row.title, "kind": kind_word(i)}))
        .collect();
    let value = json!({
        "id": format_ulid(stack.uid), "key": stack.key, "title": stack.title,
        "folder": stack.folder, "parent": stack.parent, "subjects": stack.subjects,
        "scheduler": stack.scheduler.map(|s| format!("{s:?}").to_lowercase()),
        "daily_new_limit": stack.daily_new_limit, "daily_review_limit": stack.daily_review_limit,
        "familiarity_percent": familiarity, "items": items,
    });
    Ok(json_content(uri, &value))
}

/// The item named by its id or its key (`FLASHCARD_12`, ignoring case).
pub(crate) fn find_item<'a>(items: &'a [LibraryItem], wanted: &str) -> Option<&'a LibraryItem> {
    let uid = parse_ulid(wanted);
    items.iter().find(|i| {
        Some(i.uid) == uid
            || i.row
                .key
                .as_deref()
                .is_some_and(|k| k.eq_ignore_ascii_case(wanted))
    })
}

fn schedule_json(s: &ItemSchedule) -> Value {
    let state = match s.state {
        ScheduleState::Fsrs {
            stability,
            difficulty,
        } => json!({"algorithm": "fsrs", "stability_days": stability, "difficulty": difficulty}),
        ScheduleState::Sm2 {
            ease,
            repetitions,
            interval_days,
        } => {
            json!({"algorithm": "sm2", "ease": ease, "repetitions": repetitions, "interval_days": interval_days})
        }
    };
    json!({
        "stack": format_ulid(s.stack), "folder": s.folder, "due_ms": s.due_ms,
        "last_review_ms": s.last_review_ms, "state": state,
    })
}

fn read_item(
    ctx: &mut VaultCtx,
    uri: &str,
    parts: &[(String, String)],
) -> Result<McpResourceContent, McpError> {
    let wanted = part(parts, "id");
    let items = rows_of(ctx)?;
    let item = find_item(&items, wanted)
        .ok_or_else(|| mcp_error(RESOURCE_NOT_FOUND, format!("no item {wanted}")))?;
    let root = &ctx.session.root;
    let full = root.join(&item.row.path);
    let inside =
        matches!((full.canonicalize(), root.canonicalize()), (Ok(f), Ok(b)) if f.starts_with(&b));
    if !inside {
        return Err(invalid("the item's file is not inside the vault"));
    }
    let text = fs::read_to_string(&full).map_err(internal)?;
    let schedules = item_schedules(&ctx.session.conn, item.uid).map_err(internal)?;
    let schedule = json!({
        "id": format_ulid(item.uid), "key": item.row.key, "path": item.row.path,
        "schedules": schedules.iter().map(schedule_json).collect::<Vec<_>>(),
    });
    Ok(McpResourceContent {
        uri: uri.to_owned(),
        mime_type: "text/markdown",
        text,
        more: vec![json_content(&format!("{uri}#schedule"), &schedule)],
    })
}

/// Register the vault resources.
pub fn register(mut reg: McpRegistry<VaultCtx>) -> McpRegistry<VaultCtx> {
    for (uri_template, name, description, mime_type, read) in [
        (
            "studyfrog://vault/summary",
            "Vault summary",
            "The vault's name, its stacks with item, due and new counts, items by kind, the due count and the streak.",
            "application/json",
            read_summary as studyfrog_agent::mcp_types::McpReadFn<VaultCtx>,
        ),
        (
            "studyfrog://stacks/{stack}",
            "Stack",
            "A stack by id, folder (%2F for /) or title: settings, familiarity and its items.",
            "application/json",
            read_stack,
        ),
        (
            "studyfrog://items/{id}",
            "Item",
            "An item by id or key: its Markdown file, then its FSRS or SM-2 schedule in every stack as JSON.",
            "text/markdown",
            read_item,
        ),
    ] {
        reg = register_resource(
            reg,
            McpResourceTemplate {
                uri_template,
                name,
                description,
                mime_type,
                read,
            },
        );
    }
    reg
}
