//! The user's help pages in the assistant: as context next to the stacks (within the same budget), and
//! applying the page changes of a proposal the user accepted.

use std::path::Path;

use studyfrog_agent::wiki::{Change, ContextItem};
use studyfrog_agent::wiki_pages::page_id;

use crate::agent_vault::{CONTEXT_BUDGET, ITEM_BUDGET, WikiContext};
use crate::help::{UserPage, user_title};
use crate::help_user::{list_user_pages, save_user_page};
use crate::session::ServiceError;

/// `ctx` with the user's help pages added, as many as fit; the rest count as left out.
pub fn with_help_pages(ctx: WikiContext, pages: &[UserPage]) -> WikiContext {
    let mut ctx = ctx;
    for page in pages {
        let text: String = page.text.chars().take(ITEM_BUDGET).collect();
        let n = text.chars().count();
        if ctx.chars + n > CONTEXT_BUDGET {
            ctx.left_out += 1;
            continue;
        }
        ctx.chars += n;
        ctx.items.push(ContextItem {
            uid: page_id(&page.file),
            kind: "help page".to_owned(),
            title: user_title(page).to_owned(),
            stack: String::new(),
            text,
        });
    }
    ctx
}

/// Apply a page change: a new page must not exist yet, a changed one must. Other changes are ignored.
pub fn apply_page(root: &Path, change: &Change) -> Result<(), ServiceError> {
    let exists = |name: &str| -> Result<bool, ServiceError> {
        Ok(list_user_pages(root)?.iter().any(|p| p.file == name))
    };
    match change {
        Change::CreatePage { name, text } => save_user_page(root, None, name, text).map(|_| ()),
        Change::UpdatePage { name, text } if exists(name)? => {
            save_user_page(root, Some(name), name, text).map(|_| ())
        }
        Change::UpdatePage { name, .. } => Err(ServiceError::Invalid(format!(
            "your help page \"{name}\" no longer exists"
        ))),
        _ => Ok(()),
    }
}
