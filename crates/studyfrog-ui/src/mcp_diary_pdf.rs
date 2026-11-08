//! The learning diary resource and the PDF reading tool of the MCP server (STORY-0118):
//! `studyfrog://diary/{date}` and `pdf_read_pages`. Read-only.

use serde_json::{Value, json};
use studyfrog_agent::mcp_types::{
    McpContent, McpError, McpRegistry, McpResourceContent, McpResourceTemplate, McpTool,
    register_resource, register_tool,
};
use studyfrog_core::civil::{format_date, parse_date};
use studyfrog_core::diary::RatingValue;
use studyfrog_md::ulid::format_ulid;
use studyfrog_service::pdf_marks::load_marks;
use studyfrog_store::diary::list_diary_entries_by_day;
use studyfrog_store::pdf_index::{list_pdf_rows, pdf_page_plain};

use crate::mcp_vault::{
    VaultCtx, arg_count, arg_str, internal, invalid, json_content, json_result, part, read_only,
    tool_error,
};

/// Most pages one call reads.
pub const MAX_PAGES: usize = 20;
/// Most characters of one page's text.
const PAGE_CHARS: usize = 12_000;

fn rating(v: &RatingValue) -> Value {
    match v {
        RatingValue::Int(n) => json!(n),
        RatingValue::Text(t) => json!(t),
    }
}

fn read_diary(
    ctx: &mut VaultCtx,
    uri: &str,
    parts: &[(String, String)],
) -> Result<McpResourceContent, McpError> {
    let text = part(parts, "date");
    let date = parse_date(text).ok_or_else(|| invalid("the date is written YYYY-MM-DD"))?;
    let entries = list_diary_entries_by_day(&ctx.session.conn, date).map_err(internal)?;
    let list: Vec<Value> = entries
        .iter()
        .map(|e| {
            let ratings: serde_json::Map<String, Value> = e
                .ratings
                .iter()
                .map(|(k, v)| (k.clone(), rating(v)))
                .collect();
            json!({
                "id": format_ulid(e.uid), "path": e.path, "title": e.title,
                "study_minutes": e.study_minutes, "ratings": ratings, "text": e.content,
                "stacks": e.stacks, "tags": e.tags,
                "runs": e.runs.iter().map(|u| format_ulid(*u)).collect::<Vec<_>>(),
                "items": e.items.iter().map(|u| format_ulid(*u)).collect::<Vec<_>>(),
            })
        })
        .collect();
    let value = json!({"date": format_date(date), "entries": list,
        "note": "Ratings are the 1 to 5 self-assessment scales (and custom ones) of each entry."});
    Ok(json_content(uri, &value))
}

fn read_pages(ctx: &mut VaultCtx, args: &Value) -> Result<McpContent, McpError> {
    let path = arg_str(args, "path").ok_or_else(|| invalid("give the PDF's path in the vault"))?;
    let conn = &ctx.session.conn;
    let Some(row) = list_pdf_rows(conn)
        .map_err(internal)?
        .into_iter()
        .find(|r| r.path == path)
    else {
        return Ok(tool_error(format!(
            "{path} is not a PDF in the vault's index"
        )));
    };
    let last_page = row.pages.max(1);
    let first = arg_count(args, "first_page", 1, (1, last_page));
    let last = arg_count(args, "last_page", first, (first, last_page)).min(first + MAX_PAGES - 1);
    let marks = load_marks(&ctx.session, path).unwrap_or_default();
    let mut pages = Vec::new();
    for page in first..=last {
        let plain = pdf_page_plain(conn, path, page - 1).map_err(internal)?;
        let text: Option<String> = plain.map(|p| p.text.chars().take(PAGE_CHARS).collect());
        let notes: Vec<Value> = marks
            .iter()
            .filter(|m| m.page + 1 == page)
            .map(|m| json!({"id": m.id, "kind": format!("{:?}", m.kind).to_lowercase(), "text": m.covered, "note": m.note}))
            .collect();
        pages.push(json!({"page": page, "text": text, "annotations": notes}));
    }
    Ok(json_result(&json!({
        "path": row.path, "title": row.title, "pages_in_file": row.pages, "pages": pages,
    })))
}

/// Register the diary resource and the PDF tool.
pub fn register(reg: McpRegistry<VaultCtx>) -> McpRegistry<VaultCtx> {
    let reg = register_resource(
        reg,
        McpResourceTemplate {
            uri_template: "studyfrog://diary/{date}",
            name: "Diary day",
            description: "The learning diary entries of one day (YYYY-MM-DD) with their self-assessment ratings.",
            mime_type: "application/json",
            read: read_diary,
        },
    );
    register_tool(
        reg,
        McpTool {
            name: "pdf_read_pages",
            description: "The extracted text and the highlights and notes of pages of a PDF in the vault (1-based, at most 20 pages per call).",
            input_schema: r#"{"type":"object","properties":{"path":{"type":"string","description":"The PDF's path in the vault, such as papers/a.pdf"},"first_page":{"type":"integer","minimum":1,"default":1},"last_page":{"type":"integer","minimum":1}},"required":["path"]}"#,
            annotations: read_only("Read PDF pages"),
            staged: false,
            call: read_pages,
        },
    )
}
