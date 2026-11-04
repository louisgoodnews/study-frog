//! Import: writing a bundle into the vault after the dry run (`docs/specs/interchange.md`).

use studyfrog_core::event::difficulty_from_name;
use studyfrog_core::plan::ItemKind;
use studyfrog_core::priority::priority_from_name;
use studyfrog_core::review::Review;
use studyfrog_core::status::status_from_name;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::doc::{CardDoc, Doc, NoteDoc, QuestionDoc, StackDoc};
use studyfrog_md::extra::with_extra;
use studyfrog_md::render::render_doc;
use studyfrog_store::reviews::insert_review;
use studyfrog_store::schedule::save_mem_state;
use studyfrog_store::stackmap::STACK_FILE;

use crate::create::{IdSource, meta, refresh, render_error, slug, unique_name, write_new};
use crate::import::nth;
use crate::rerate::replay_fsrs;
use crate::session::{ServiceError, VaultInfo, VaultSession, current_settings};
use crate::settings::params_for;
use crate::xfer_model::{Bundle, XItem};
pub use crate::xfer_plan::{ImportPlan, MAX_ITEMS, PlanLine, Policy, Verdict, counts, plan_import};
use crate::xfer_plan::{find, known, label};

fn doc_of(item: &XItem, uid: Uid, ids: IdSource, notes: &mut Vec<String>) -> Doc {
    let title = label(item);
    let mut m = meta(&title, &item.tags, ids);
    m.uid = uid;
    let mut doc = match item.kind {
        ItemKind::Note => Doc::Note(NoteDoc {
            meta: m,
            rehearse_title: false,
            body: item.text.clone(),
            extra: Vec::new(),
        }),
        ItemKind::Card => Doc::Card(CardDoc {
            meta: m,
            reversible: false,
            front: item.text.clone(),
            back: item.answer.clone(),
            extra: Vec::new(),
        }),
        ItemKind::Question => Doc::Question(QuestionDoc {
            meta: m,
            prompt: item.text.clone(),
            reference: item.answer.clone(),
            accepted: Vec::new(),
            rubric: Vec::new(),
            min_score: studyfrog_core::items::DEFAULT_MIN_SCORE,
            extra: Vec::new(),
        }),
    };
    for (key, word, valid) in [
        (
            "status",
            &item.status,
            item.status
                .as_deref()
                .is_some_and(|w| status_from_name(w).is_some()),
        ),
        (
            "priority",
            &item.priority,
            item.priority
                .as_deref()
                .is_some_and(|w| priority_from_name(w).is_some()),
        ),
        (
            "difficulty",
            &item.difficulty,
            item.difficulty
                .as_deref()
                .is_some_and(|w| difficulty_from_name(w).is_some()),
        ),
    ] {
        match (word, valid) {
            (Some(w), true) => doc = with_extra(&doc, key, Some(w)),
            (Some(w), false) => notes.push(format!("{}: \"{w}\" is not a {key}", label(item))),
            _ => {}
        }
    }
    for (key, value) in &item.extra {
        doc = with_extra(&doc, key, Some(value));
    }
    if item.id.is_some() {
        doc
    } else {
        crate::create::with_new_defaults(doc)
    }
}

pub(crate) fn ensure_stacks(
    session: &VaultSession,
    folder: &str,
    b: &Bundle,
    ids: IdSource,
    n: &mut usize,
) -> Result<(), ServiceError> {
    let mut upto = String::new();
    for part in folder.split('/') {
        upto = if upto.is_empty() {
            part.to_owned()
        } else {
            format!("{upto}/{part}")
        };
        let file = session.root.join(&upto).join(STACK_FILE);
        if file.is_file() {
            continue;
        }
        let info = b.stacks.iter().find(|s| s.folder == upto);
        let title = info.map_or_else(|| part.to_owned(), |s| s.title.clone());
        *n += 1;
        let doc = Doc::Stack(StackDoc {
            meta: meta(&title, &[], nth(ids, 10_000 + *n)),
            description: info.map_or_else(String::new, |s| s.description.clone()),
            body: String::new(),
            extra: Vec::new(),
        });
        write_new(&file, &render_doc(&doc).map_err(render_error)?)?;
    }
    Ok(())
}

/// What an import wrote.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportDone {
    /// Items written as new.
    pub created: usize,
    /// Existing items overwritten.
    pub replaced: usize,
    /// Items left out (duplicates skipped, errors).
    pub skipped: usize,
    /// Things worth telling the user.
    pub notes: Vec<String>,
    /// The vault afterwards.
    pub info: VaultInfo,
}

fn insert_reviews(session: &VaultSession, uid: Uid, item: &XItem) {
    let reviews: Vec<Review> = item
        .reviews
        .iter()
        .map(|r| Review {
            item: uid,
            rating: r.rating,
            at: UnixMs(r.at_ms),
            elapsed_days: r.elapsed_days,
        })
        .collect();
    for r in &reviews {
        let _ = insert_review(&session.conn, r);
    }
    let params = params_for(&current_settings(session), None);
    if let Some(mem) = replay_fsrs(&reviews, &params) {
        let _ = save_mem_state(&session.conn, uid, &mem);
    }
}

/// Write the bundle: new items are created, duplicates are handled by `policy`, and nothing is written for items with errors.
pub fn apply_import(
    session: &mut VaultSession,
    b: &Bundle,
    target: Option<&str>,
    (policy, with_schedule): (Policy, bool),
    ids: IdSource,
) -> Result<ImportDone, ServiceError> {
    let plan = plan_import(session, b, target)?;
    let k = known(session)?;
    let (mut created, mut replaced, mut skipped, mut stacks) = (0, 0, 0, 0);
    let mut notes = plan.warnings.clone();
    let mut named: Vec<crate::xfer_refs::Named> = Vec::new();
    let mut histories: Vec<(Uid, &XItem)> = Vec::new();
    for (n, (item, line)) in b.items.iter().zip(&plan.lines).enumerate() {
        let (uid, path) = match (&line.verdict, policy) {
            (Verdict::Error(e), _) => {
                notes.push(format!("{}: {e}", line.title));
                skipped += 1;
                continue;
            }
            (Verdict::Duplicate(_), Policy::Skip) => {
                skipped += 1;
                continue;
            }
            (Verdict::Duplicate(p), Policy::Replace) => (
                find(&k, item, &line.folder).map_or(Uid(0), |e| e.0),
                Some(p.clone()),
            ),
            (Verdict::Duplicate(_), Policy::KeepBoth) => {
                (crate::xfer_import::fresh_uid(ids, n), None)
            }
            (Verdict::Create, _) => (item.id.unwrap_or_else(|| fresh_uid(ids, n)), None),
        };
        ensure_stacks(session, &line.folder, b, ids, &mut stacks)?;
        let text = render_doc(&doc_of(item, uid, nth(ids, n), &mut notes)).map_err(render_error)?;
        let file = match &path {
            Some(p) => p.clone(),
            None => {
                let dir = session.root.join(&line.folder);
                format!(
                    "{}/{}",
                    line.folder,
                    unique_name(&slug(&label(item)), ".md", |c| dir.join(c).exists())
                )
            }
        };
        write_new(&session.root.join(&file), &text)?;
        if !item.also_in.is_empty() {
            named.push((file.clone(), item.also_in.clone()));
        }
        if path.is_some() {
            replaced += 1;
        } else {
            created += 1;
            if with_schedule && !item.reviews.is_empty() {
                histories.push((uid, item));
            }
        }
    }
    crate::xfer_files::copy_files(session, b);
    notes.extend(crate::xfer_refs::link_references(
        session,
        (&named, b),
        ids,
        &mut stacks,
    )?);
    let info = refresh(session)?;
    // After the reindex, so each history lands in the stack the item now lives in.
    for (uid, item) in histories {
        insert_reviews(session, uid, item);
    }
    Ok(ImportDone {
        created,
        replaced,
        skipped,
        notes,
        info,
    })
}

fn fresh_uid(ids: IdSource, n: usize) -> Uid {
    studyfrog_md::ulid::compose_ulid(ids.now_ms, nth(ids, n).random)
}
