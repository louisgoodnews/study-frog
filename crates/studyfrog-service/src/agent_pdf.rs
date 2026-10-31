//! PDF pages as context for the assistant: the text of chosen pages, each with an id of its own so
//! that a card or question the model proposes can name the page it comes from.

use studyfrog_agent::wiki::{ContextItem, PDF_PREFIX};
use studyfrog_store::pdf_index::{list_pdf_rows, pdf_page_plain};

use crate::agent_vault::{CONTEXT_BUDGET, ITEM_BUDGET, WikiContext, wiki_context};
use crate::session::{ServiceError, VaultSession};

/// Most pages of one PDF sent in one request.
pub const MAX_PDF_PAGES: usize = 20;

/// The id the model sees for a page (1-based in the text).
pub fn page_id(pdf: &str, page: usize) -> String {
    format!("{PDF_PREFIX}{pdf}#page={}", page + 1)
}

/// The context for the chosen stacks plus the text of pages `first` to `last` (0-based) of a PDF.
/// Pages without text are skipped; pages beyond the limits count as left out.
pub fn wiki_context_with_pdf(
    session: &VaultSession,
    folders: &[String],
    pdf: Option<(&str, usize, usize)>,
) -> Result<WikiContext, ServiceError> {
    let mut ctx = wiki_context(session, folders)?;
    let Some((path, first, last)) = pdf else {
        return Ok(ctx);
    };
    let store = ServiceError::Store;
    let title = list_pdf_rows(&session.conn)
        .map_err(store)?
        .into_iter()
        .find(|r| r.path == path)
        .map(|r| r.title)
        .ok_or_else(|| ServiceError::Invalid("that PDF is not in the index yet".to_owned()))?;
    for page in first..=last.max(first) {
        let text = pdf_page_plain(&session.conn, path, page).map_err(ServiceError::Store)?;
        let Some(text) = text.map(|p| p.text).filter(|t| !t.trim().is_empty()) else {
            continue;
        };
        let text: String = text.chars().take(ITEM_BUDGET).collect();
        let over =
            page - first >= MAX_PDF_PAGES || ctx.chars + text.chars().count() > CONTEXT_BUDGET;
        if over {
            ctx.left_out += 1;
            continue;
        }
        ctx.chars += text.chars().count();
        ctx.items.push(ContextItem {
            uid: page_id(path, page),
            kind: "pdf".to_owned(),
            title: format!("{title}, page {}", page + 1),
            stack: String::new(),
            text,
        });
    }
    Ok(ctx)
}
