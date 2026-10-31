//! What the agent reads from the vault and what it may write back: the context for a wiki session or
//! an exam, applying a proposal the user accepted, and recording exam verdicts as reviews.

use studyfrog_agent::exam::{ExamItem, ExamResult, Verdict};
use studyfrog_agent::wiki::ContextItem;
use studyfrog_core::plan::{ItemKind, item_kind_name};
use studyfrog_core::review::Rating;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::doc::Doc;
use studyfrog_md::ulid::{format_ulid, parse_ulid};
use studyfrog_store::home::{IndexRow, item_index};
use studyfrog_store::overview::load_overview;

use crate::content::{ItemContent, load_doc, load_item_content};
use crate::session::{ServiceError, VaultSession, current_settings};

/// Most characters of item text sent in one request.
pub const CONTEXT_BUDGET: usize = 30_000;
/// Most characters of one item.
pub const ITEM_BUDGET: usize = 3_000;

/// What a wiki request will contain.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WikiContext {
    /// The stacks as (folder, title).
    pub stacks: Vec<(String, String)>,
    /// The items.
    pub items: Vec<ContextItem>,
    /// Items left out for lack of room.
    pub left_out: usize,
    /// Characters of item text.
    pub chars: usize,
}

fn inside(folder: &str, outer: &str) -> bool {
    folder == outer || folder.starts_with(&format!("{outer}/"))
}

fn text_of(kind: ItemKind, c: &ItemContent) -> String {
    let text = match kind {
        ItemKind::Note => c.answer.clone(),
        _ => format!("Front: {}\nBack: {}", c.prompt, c.answer),
    };
    text.chars().take(ITEM_BUDGET).collect()
}

/// The text of every item in the chosen stacks (and the stacks inside them), as much as fits.
pub fn wiki_context(
    session: &VaultSession,
    folders: &[String],
) -> Result<WikiContext, ServiceError> {
    let store = ServiceError::Store;
    let stacks: Vec<(String, String)> = load_overview(&session.conn)
        .map_err(store)?
        .stacks
        .into_iter()
        .filter(|s| folders.iter().any(|f| inside(&s.folder, f)))
        .map(|s| (s.folder, s.title))
        .collect();
    let mut out = WikiContext {
        stacks,
        ..WikiContext::default()
    };
    let rows: Vec<IndexRow> = item_index(&session.conn)
        .map_err(store)?
        .into_iter()
        .filter(|i| {
            i.stack
                .as_deref()
                .is_some_and(|s| out.stacks.iter().any(|(f, _)| f == s))
        })
        .collect();
    for row in rows {
        let content = load_item_content(session, row.uid)?;
        let text = text_of(row.kind, &content);
        if out.chars + text.chars().count() > CONTEXT_BUDGET {
            out.left_out += 1;
            continue;
        }
        out.chars += text.chars().count();
        out.items.push(ContextItem {
            uid: format_ulid(row.uid),
            kind: item_kind_name(row.kind).to_owned(),
            title: row.title,
            stack: row.stack.unwrap_or_default(),
            text,
        });
    }
    Ok(out)
}

/// Up to `count` cards and questions of the chosen stacks, in vault order, for an exam.
pub fn exam_items(
    session: &VaultSession,
    folders: &[String],
    count: usize,
) -> Result<Vec<ExamItem>, ServiceError> {
    let rows = item_index(&session.conn).map_err(ServiceError::Store)?;
    let mut out = Vec::new();
    for row in rows.into_iter().filter(|r| {
        r.kind != ItemKind::Note
            && r.stack
                .as_deref()
                .is_some_and(|s| folders.iter().any(|f| inside(s, f)))
    }) {
        if out.len() >= count {
            break;
        }
        let c = match load_doc(session, row.uid)? {
            Doc::Card(d) => (d.front, d.back),
            Doc::Question(d) => (d.prompt, d.reference),
            _ => continue,
        };
        out.push(ExamItem {
            uid: format_ulid(row.uid),
            prompt: c.0,
            reference: c.1,
        });
    }
    Ok(out)
}

/// The rating an exam verdict stands for.
pub fn rating_of(v: Verdict) -> Rating {
    match v {
        Verdict::Correct => Rating::Good,
        Verdict::Partial => Rating::Hard,
        Verdict::Incorrect => Rating::Again,
    }
}

/// Record the verdicts as reviews (the same way a rehearsal run does); returns how many.
pub fn record_exam(
    session: &mut VaultSession,
    results: &[ExamResult],
    now: UnixMs,
) -> Result<u32, ServiceError> {
    let vault = current_settings(session);
    let tx = session
        .conn
        .transaction()
        .map_err(|e| ServiceError::Invalid(e.to_string()))?;
    let mut n = 0;
    for r in results {
        let Some(uid) = parse_ulid(&r.uid) else {
            continue;
        };
        crate::rehearse::rate_item(&tx, uid, rating_of(r.verdict), now, &vault)
            .map_err(ServiceError::Store)?;
        n += 1;
    }
    tx.commit()
        .map_err(|e| ServiceError::Invalid(e.to_string()))?;
    let _ = Uid(0);
    Ok(n)
}

pub use crate::agent_apply::{Applied, apply_changes};
