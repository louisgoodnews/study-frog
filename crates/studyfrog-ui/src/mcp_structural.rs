//! The structural tools of the MCP server (STORY-0119): `vault_move_stack` and `vault_delete_item`.
//! They are destructive, so they are listed and run only when the server was started with
//! `--mcp-allow-write --mcp-allow-destructive`; each keeps a full undo entry the user can take back
//! in the Assistant.

use serde_json::{Value, json};
use studyfrog_agent::mcp_types::{
    McpContent, McpError, McpRegistry, McpTool, McpToolAnnotations, register_tool,
};
use studyfrog_md::ulid::{compose_ulid, format_ulid};
use studyfrog_service::create::fresh_id_source;
use studyfrog_service::folder_ops::{FolderSnapshot, delete_path, move_path};
use studyfrog_service::mcp_undo::{UndoEntry, record_undo};

use crate::mcp_vault::{
    VaultCtx, arg_str, internal, invalid, json_result, now_of, rows_of, tool_error,
};
use crate::mcp_vault_read::{find_item, find_stack};
use crate::mcp_write::stacks;

const fn destructive(title: &'static str) -> McpToolAnnotations {
    McpToolAnnotations {
        title,
        read_only_hint: false,
        destructive_hint: true,
        idempotent_hint: false,
        open_world_hint: false,
    }
}

fn keep_undo(ctx: &VaultCtx, label: String, snapshot: FolderSnapshot) -> Result<String, McpError> {
    let now = now_of(ctx);
    let ids = fresh_id_source(u64::try_from(now.0).unwrap_or(0)).map_err(internal)?;
    let id = format_ulid(compose_ulid(ids.now_ms, ids.random));
    let entry = UndoEntry {
        id: id.clone(),
        label,
        at_ms: now.0,
        snapshot,
    };
    record_undo(&ctx.session.root, &entry).map_err(internal)?;
    Ok(id)
}

fn move_stack(ctx: &mut VaultCtx, args: &Value) -> Result<McpContent, McpError> {
    let wanted = arg_str(args, "stack").ok_or_else(|| invalid("name the stack to move"))?;
    let all = stacks(ctx)?;
    let Some(stack) = find_stack(&all, wanted) else {
        return Ok(tool_error(format!("there is no stack {wanted}")));
    };
    let into = match arg_str(args, "into") {
        None => String::new(),
        Some(parent) => match find_stack(&all, parent) {
            Some(p) => p.folder.clone(),
            None => return Ok(tool_error(format!("there is no stack {parent}"))),
        },
    };
    let src = stack.folder.clone();
    let (_, snapshot, plan) = match move_path(&mut ctx.session, &src, &into) {
        Ok(done) => done,
        Err(e) => return Ok(tool_error(e.to_string())),
    };
    let label = format!("Moved the stack {} to {}", plan.src, plan.dst);
    let undo = keep_undo(ctx, label, snapshot)?;
    Ok(json_result(&json!({
        "moved": plan.src, "to": plan.dst, "undo": undo,
        "schedules_kept": plan.schedules_dropped == 0,
        "note": "The user can undo this in StudyFrog: Assistant > Work the vault > Changes by AI tools.",
    })))
}

fn delete_item(ctx: &mut VaultCtx, args: &Value) -> Result<McpContent, McpError> {
    let wanted = arg_str(args, "item").ok_or_else(|| invalid("name the item to delete"))?;
    let items = rows_of(ctx)?;
    let Some(item) = find_item(&items, wanted) else {
        return Ok(tool_error(format!("there is no item {wanted}")));
    };
    let (path, title) = (item.row.path.clone(), item.row.title.clone());
    let (_, snapshot, _) = match delete_path(&mut ctx.session, &path) {
        Ok(done) => done,
        Err(e) => return Ok(tool_error(e.to_string())),
    };
    let undo = keep_undo(ctx, format!("Deleted \"{title}\" ({path})"), snapshot)?;
    Ok(json_result(&json!({
        "deleted": path, "undo": undo,
        "note": "The file is kept in the undo entry; the user can restore it in StudyFrog: Assistant > Work the vault > Changes by AI tools.",
    })))
}

/// Register the structural tools.
pub fn register(reg: McpRegistry<VaultCtx>) -> McpRegistry<VaultCtx> {
    let reg = register_tool(
        reg,
        McpTool {
            name: "vault_move_stack",
            description: "Move a stack into another stack, or out to the top level when `into` is omitted, keeping every item's schedule. Needs a server started with --mcp-allow-write --mcp-allow-destructive; keeps an undo entry.",
            input_schema: r#"{"type":"object","properties":{"stack":{"type":"string","description":"Stack folder, id or title"},"into":{"type":"string","description":"The new parent stack; omit for top level"}},"required":["stack"]}"#,
            annotations: destructive("Move a stack"),
            staged: false,
            call: move_stack,
        },
    );
    register_tool(
        reg,
        McpTool {
            name: "vault_delete_item",
            description: "Delete an item (by id or key). Needs a server started with --mcp-allow-write --mcp-allow-destructive; the whole file is kept in an undo entry.",
            input_schema: r#"{"type":"object","properties":{"item":{"type":"string","description":"The item's id or key"}},"required":["item"]}"#,
            annotations: destructive("Delete an item"),
            staged: false,
            call: delete_item,
        },
    )
}
