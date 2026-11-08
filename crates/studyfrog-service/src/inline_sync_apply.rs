//! Carrying out a [`SyncPlan`]: new blocks become cards or questions in the note's stack, changed
//! answers are written into the items that have them. Schedules are kept: an item keeps its id.

use std::fs;

use studyfrog_core::items::DEFAULT_MIN_SCORE;
use studyfrog_md::doc::{CardDoc, Doc, QuestionDoc};
use studyfrog_md::extra::with_extra;
use studyfrog_md::inline_extract::{InlineKind, extract_key};
use studyfrog_md::parse::parse_doc;
use studyfrog_md::render::render_doc;
use studyfrog_md::ulid::format_ulid;

use crate::create::{
    IdSource, default_card_title, meta, refresh, render_error, slug, unique_name,
    with_new_defaults, write_new,
};
use crate::inline_sync::{FROM_FIELD, KEY_FIELD, SyncChange, SyncEntry, SyncPlan, stamp};
use crate::session::{ServiceError, VaultInfo, VaultSession};

/// What a sync did.
#[derive(Debug, Clone, PartialEq)]
pub struct SyncOutcome {
    /// The vault after reindexing.
    pub info: VaultInfo,
    /// Items created.
    pub created: usize,
    /// Items whose answer was replaced.
    pub updated: usize,
    /// How to take the sync back (the bulk edit's undo: new files removed, old texts restored).
    pub undo: crate::bulk::Snapshot,
}

fn new_doc(plan: &SyncPlan, entry: &SyncEntry, ids: IdSource) -> Doc {
    let b = &entry.block;
    let m = meta(&default_card_title(&b.front), &plan.tags, ids);
    let doc = match b.kind {
        InlineKind::Card => Doc::Card(CardDoc {
            meta: m,
            reversible: false,
            front: b.front.clone(),
            back: entry.answer.clone(),
            extra: Vec::new(),
        }),
        InlineKind::Question => Doc::Question(QuestionDoc {
            meta: m,
            prompt: b.front.clone(),
            reference: entry.answer.clone(),
            accepted: Vec::new(),
            rubric: Vec::new(),
            min_score: DEFAULT_MIN_SCORE,
            extra: Vec::new(),
        }),
    };
    let doc = with_extra(&doc, FROM_FIELD, Some(&format_ulid(plan.note)));
    with_new_defaults(with_extra(&doc, KEY_FIELD, Some(&extract_key(b))))
}

fn create_one(
    session: &VaultSession,
    plan: &SyncPlan,
    entry: &SyncEntry,
    ids: IdSource,
) -> Result<String, ServiceError> {
    let doc = crate::item_keys::with_new_key(&session.conn, new_doc(plan, entry, ids))?;
    let text = render_doc(&doc).map_err(render_error)?;
    let dir = session.root.join(&plan.folder);
    let base = slug(&studyfrog_md::doc::doc_meta(&doc).title);
    let name = unique_name(&base, ".md", |n| dir.join(n).exists());
    write_new(&dir.join(&name), &text)?;
    Ok(format!("{}/{name}", plan.folder))
}

fn update_one(
    session: &VaultSession,
    path: &str,
    answer: &str,
    now_ms: u64,
) -> Result<String, ServiceError> {
    let abs = session.root.join(path);
    let io = |e: std::io::Error| ServiceError::Io(format!("{path}: {e}"));
    let text = fs::read_to_string(&abs).map_err(io)?;
    let mut doc = parse_doc(&text).map_err(|e| ServiceError::Invalid(format!("{path}: {e}")))?;
    match &mut doc {
        Doc::Card(c) => {
            c.back = answer.to_owned();
            c.meta.updated = Some(stamp(now_ms));
        }
        Doc::Question(q) => {
            q.reference = answer.to_owned();
            q.meta.updated = Some(stamp(now_ms));
        }
        _ => return Err(ServiceError::Invalid(format!("{path} is not a card"))),
    }
    write_new(&abs, &render_doc(&doc).map_err(render_error)?)?;
    Ok(text)
}

/// Create the new items and update the changed ones, then reindex once. `ids` gives one id per new
/// item (the random bytes of each are varied by its position).
pub fn apply_inline_sync(
    session: &mut VaultSession,
    plan: &SyncPlan,
    ids: IdSource,
) -> Result<SyncOutcome, ServiceError> {
    let (mut created, mut updated, mut undo) = (0, 0, Vec::new());
    for (n, entry) in plan.entries.iter().enumerate() {
        match (entry.change, &entry.existing) {
            (SyncChange::New, _) => {
                let mut random = ids.random;
                random[9] = random[9].wrapping_add(n as u8);
                random[8] = random[8].wrapping_add((n >> 8) as u8);
                let path = create_one(session, plan, entry, IdSource { random, ..ids })?;
                undo.push((path, None));
                created += 1;
            }
            (SyncChange::Updated, Some((_, path))) => {
                let old = update_one(session, path, &entry.answer, ids.now_ms)?;
                undo.push((path.clone(), Some(old)));
                updated += 1;
            }
            _ => {}
        }
    }
    Ok(SyncOutcome {
        info: refresh(session)?,
        created,
        updated,
        undo: crate::bulk::Snapshot(undo),
    })
}
