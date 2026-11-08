//! `note_extract_inline_cards` (STORY-0119): the inline cards of a note become proposed cards and
//! questions in the note's stack. Blocks that already exist there as items are skipped, so running
//! it again after applying proposes nothing new.

use serde_json::{Value, json};
use studyfrog_agent::mcp_types::{McpContent, McpError};
use studyfrog_agent::wiki::Change;
use studyfrog_core::plan::ItemKind;
use studyfrog_md::doc::Doc;
use studyfrog_md::inline_extract::{InlineBlock, InlineKind, extract_inline};
use studyfrog_service::document::load_document;
use studyfrog_service::library_rows::LibraryItem;

use crate::mcp_vault::{VaultCtx, arg_str, internal, invalid, json_result, rows_of, tool_error};
use crate::mcp_vault_read::find_item;
use crate::mcp_write::{stacks, stage_or_apply};

/// True when a card or question of the stack already holds this block's front and back.
fn exists(items: &[LibraryItem], folder: &str, b: &InlineBlock) -> bool {
    items.iter().any(|i| {
        matches!(i.row.kind, ItemKind::Card | ItemKind::Question)
            && i.row.path.starts_with(&format!("{folder}/"))
            && i.row.body.contains(&b.front)
            && i.row.body.contains(&b.back)
    })
}

fn change_of(folder: &str, b: &InlineBlock) -> Change {
    match b.kind {
        InlineKind::Card => Change::CreateCard {
            stack: folder.to_owned(),
            front: b.front.clone(),
            back: b.back.clone(),
            source: None,
        },
        InlineKind::Question => Change::CreateQuestion {
            stack: folder.to_owned(),
            prompt: b.front.clone(),
            reference: b.back.clone(),
            source: None,
        },
    }
}

/// The tool.
pub fn extract(ctx: &mut VaultCtx, args: &Value) -> Result<McpContent, McpError> {
    let wanted = arg_str(args, "note").ok_or_else(|| invalid("give the note's id or key"))?;
    let items = rows_of(ctx)?;
    let Some(item) = find_item(&items, wanted) else {
        return Ok(tool_error(format!("there is no item {wanted}")));
    };
    let loaded = load_document(&ctx.session, item.uid).map_err(internal)?;
    let Doc::Note(note) = &loaded.doc else {
        return Ok(tool_error(format!("{wanted} is not a note")));
    };
    let folder = loaded.path.rsplit_once('/').map_or("", |(f, _)| f);
    if !stacks(ctx)?.iter().any(|s| s.folder == folder) {
        return Ok(tool_error(
            "the note is not in a stack, so its cards have nowhere to go",
        ));
    }
    let blocks = extract_inline(&note.body);
    let (new, old): (Vec<&InlineBlock>, Vec<&InlineBlock>) =
        blocks.iter().partition(|b| !exists(&items, folder, b));
    if new.is_empty() {
        return Ok(json_result(&json!({
            "found": blocks.len(), "already_there": old.len(), "staged": Value::Null,
            "note": "No new inline cards to propose.",
        })));
    }
    let changes: Vec<Change> = new.iter().map(|b| change_of(folder, b)).collect();
    let summary = format!("Inline cards of the note \"{}\"", note.meta.title);
    let skipped = [format!("{} already in the stack were skipped.", old.len())];
    stage_or_apply(
        ctx,
        ("note_extract_inline_cards", &summary),
        changes,
        &skipped,
    )
}
