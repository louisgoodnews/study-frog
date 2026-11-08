//! The authoring tools of the MCP server (STORY-0119): `vault_propose_items`,
//! `note_extract_inline_cards`, `diary_log_entry` and `calendar_plan_session`. They check their
//! input through the service and stage the changes for the user to review in the Assistant; only a
//! server started with `--mcp-allow-write` writes them directly.

use serde_json::{Value, json};
use studyfrog_agent::mcp_stage::{diary_change_of, proposed_change, session_change_of};
use studyfrog_agent::mcp_types::{
    McpContent, McpError, McpRegistry, McpTool, McpToolAnnotations, register_tool,
};
use studyfrog_agent::wiki::Change;
use studyfrog_service::agent_vault::apply_changes;
use studyfrog_service::create::fresh_id_source;
use studyfrog_service::mcp_staging::stage_changes;
use studyfrog_store::overview::{StackRow, load_overview};

use crate::mcp_vault::{VaultCtx, arg_str, internal, json_result, now_of, tool_error};
use crate::mcp_vault_read::find_stack;

/// Most changes one call may propose.
pub const MAX_PROPOSED: usize = 200;

/// The annotations of a tool that adds to the vault and never overwrites.
pub(crate) const fn additive(title: &'static str) -> McpToolAnnotations {
    McpToolAnnotations {
        title,
        read_only_hint: false,
        destructive_hint: false,
        idempotent_hint: false,
        open_world_hint: false,
    }
}

pub(crate) fn stacks(ctx: &VaultCtx) -> Result<Vec<StackRow>, McpError> {
    Ok(load_overview(&ctx.session.conn).map_err(internal)?.stacks)
}

/// Stage the changes (or, when the server may write, apply them) and say what happened.
pub(crate) fn stage_or_apply(
    ctx: &mut VaultCtx,
    (tool, summary): (&str, &str),
    changes: Vec<Change>,
    rejected: &[String],
) -> Result<McpContent, McpError> {
    if changes.is_empty() {
        return Ok(tool_error(format!(
            "Nothing usable was proposed. {}",
            rejected.join(" ")
        )));
    }
    let now = now_of(ctx);
    let ids = fresh_id_source(u64::try_from(now.0).unwrap_or(0)).map_err(internal)?;
    if ctx.policy.allow_write {
        let done = apply_changes(&mut ctx.session, &changes, ids, now);
        return Ok(json_result(&json!({
            "applied": {"created": done.created, "updated": done.updated, "failed": done.failed},
            "rejected": rejected,
        })));
    }
    let id = stage_changes(&ctx.session.root, (tool, summary), changes.clone(), ids)
        .map_err(internal)?;
    Ok(json_result(&json!({
        "staged": id, "changes": changes.len(), "rejected": rejected,
        "note": "Nothing was written. The user reviews and applies this in StudyFrog: Assistant > Work the vault > Proposals from AI tools.",
    })))
}

/// The folder of a stack named by folder, id or title, as the change needs it.
fn folder_of(all: &[StackRow], v: &Value) -> Value {
    let mut v = v.clone();
    let named = v.get("stack").and_then(Value::as_str).map(str::to_owned);
    if let (Some(s), Some(obj)) = (named.and_then(|n| find_stack(all, &n)), v.as_object_mut()) {
        obj.insert("stack".to_owned(), json!(s.folder));
    }
    v
}

/// An item of the batch as the action object of a change, with `\n` written out as new lines.
fn item_action(v: &Value) -> Value {
    let un = |k: &str| {
        let text = v.get(k).and_then(Value::as_str).unwrap_or("");
        json!(studyfrog_service::content::unescape_newlines(text))
    };
    let kind = v.get("kind").and_then(Value::as_str).unwrap_or("card");
    match kind {
        "note" => {
            json!({"action": "create_note", "stack": v["stack"], "title": un("title"), "body": un("body")})
        }
        "question" => {
            json!({"action": "create_question", "stack": v["stack"], "prompt": un("prompt"), "reference": un("reference")})
        }
        _ => {
            json!({"action": "create_card", "stack": v["stack"], "front": un("front"), "back": un("back")})
        }
    }
}

/// The items of a StudyFrog CSV (`stack,kind,...`) as batch items.
fn csv_items(text: &str) -> Result<Vec<Value>, String> {
    let bundle = studyfrog_service::xfer_csv::from_csv(text)?;
    Ok(bundle
        .items
        .iter()
        .map(|i| {
            let kind = format!("{:?}", i.kind).to_lowercase();
            json!({"kind": kind, "stack": i.stack, "title": i.title, "front": i.text, "back": i.answer,
                "prompt": i.text, "reference": i.answer, "body": i.text})
        })
        .collect())
}

fn propose(ctx: &mut VaultCtx, args: &Value) -> Result<McpContent, McpError> {
    let all = stacks(ctx)?;
    let folders: Vec<String> = all.iter().map(|s| s.folder.clone()).collect();
    let mut items: Vec<Value> = args
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if let Some(text) = arg_str(args, "csv") {
        match csv_items(text) {
            Ok(more) => items.extend(more),
            Err(e) => return Ok(tool_error(format!("the CSV cannot be read: {e}"))),
        }
    }
    let mut rejected = Vec::new();
    if items.len() > MAX_PROPOSED {
        rejected.push(format!("Only the first {MAX_PROPOSED} items are kept."));
        items.truncate(MAX_PROPOSED);
    }
    let mut changes = Vec::new();
    for (i, item) in items.iter().enumerate() {
        match proposed_change(&item_action(&folder_of(&all, item)), &folders) {
            Ok(c) => changes.push(c),
            Err(why) => rejected.push(format!("Item {}: {why}", i + 1)),
        }
    }
    let summary = arg_str(args, "summary").unwrap_or("Items proposed by an AI client");
    stage_or_apply(ctx, ("vault_propose_items", summary), changes, &rejected)
}

fn diary(ctx: &mut VaultCtx, args: &Value) -> Result<McpContent, McpError> {
    match diary_change_of(args) {
        Ok(c) => stage_or_apply(ctx, ("diary_log_entry", "A diary entry"), vec![c], &[]),
        Err(why) => Ok(tool_error(why)),
    }
}

fn plan(ctx: &mut VaultCtx, args: &Value) -> Result<McpContent, McpError> {
    let all = stacks(ctx)?;
    let folders: Vec<String> = all.iter().map(|s| s.folder.clone()).collect();
    match session_change_of(&folder_of(&all, args), &folders) {
        Ok(c) => stage_or_apply(
            ctx,
            ("calendar_plan_session", "A planned session"),
            vec![c],
            &[],
        ),
        Err(why) => Ok(tool_error(why)),
    }
}

/// Name, description, input schema, title and function of a tool.
type ToolRow = (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    studyfrog_agent::mcp_types::McpToolFn<VaultCtx>,
);

/// Register the authoring tools.
pub fn register(mut reg: McpRegistry<VaultCtx>) -> McpRegistry<VaultCtx> {
    let tools: [ToolRow; 4] = [
        (
            "vault_propose_items",
            "Propose new cards, questions or notes, as a list of items or as StudyFrog CSV (stack,kind,front,back). They are checked and staged for the user to review; nothing is written until the user applies them (unless the server may write).",
            r#"{"type":"object","properties":{"items":{"type":"array","items":{"type":"object","properties":{"kind":{"type":"string","enum":["card","question","note"]},"stack":{"type":"string","description":"Stack folder, id or title"},"front":{"type":"string"},"back":{"type":"string"},"prompt":{"type":"string"},"reference":{"type":"string"},"title":{"type":"string"},"body":{"type":"string"}},"required":["kind","stack"]}},"csv":{"type":"string"},"summary":{"type":"string"}}}"#,
            "Propose items",
            propose,
        ),
        (
            "note_extract_inline_cards",
            "Turn the inline cards of a note (`Term :: Definition`, `Q:`/`A:` and `??:`/`A:` lines) into cards and questions in the note's stack, skipping ones that already exist. Staged for the user to review.",
            r#"{"type":"object","properties":{"note":{"type":"string","description":"The note's id or key"}},"required":["note"]}"#,
            "Extract inline cards",
            crate::mcp_extract::extract,
        ),
        (
            "diary_log_entry",
            "Record a learning diary reflection for a day with 1 to 5 self-assessment ratings (such as focus, understanding, confidence). Staged for the user to review.",
            r#"{"type":"object","properties":{"date":{"type":"string","description":"YYYY-MM-DD"},"title":{"type":"string"},"body":{"type":"string"},"study_minutes":{"type":"integer","minimum":0,"maximum":1440},"ratings":{"type":"object","additionalProperties":{"type":"integer","minimum":1,"maximum":5}},"stacks":{"type":"array","items":{"type":"string"}}},"required":["date"]}"#,
            "Log a diary entry",
            diary,
        ),
        (
            "calendar_plan_session",
            "Plan a study session (or an exam, task or appointment) on the vault's calendar on one day. Staged for the user to review.",
            r#"{"type":"object","properties":{"title":{"type":"string"},"date":{"type":"string","description":"YYYY-MM-DD"},"start":{"type":"string","description":"HH:MM local time"},"duration_minutes":{"type":"integer","minimum":1,"maximum":1440},"stack":{"type":"string","description":"Stack folder, id or title to rehearse"},"kind":{"type":"string","enum":["study","exam","task","appointment"],"default":"study"},"notes":{"type":"string"}},"required":["title","date","start","duration_minutes"]}"#,
            "Plan a session",
            plan,
        ),
    ];
    for (name, description, input_schema, title, call) in tools {
        reg = register_tool(
            reg,
            McpTool {
                name,
                description,
                input_schema,
                annotations: additive(title),
                staged: true,
                call,
            },
        );
    }
    reg
}
