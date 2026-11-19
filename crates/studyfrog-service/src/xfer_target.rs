//! Target destination resolution and new-stack creation for imports.

use studyfrog_core::event::difficulty_from_name;
use studyfrog_core::plan::ItemKind;
use studyfrog_core::priority::priority_from_name;
use studyfrog_core::review::Review;
use studyfrog_core::status::status_from_name;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::doc::{CardDoc, Doc, NoteDoc, QuestionDoc, StackDoc, doc_meta};
use studyfrog_md::extra::{extra_raw, extra_value, with_extra};
use studyfrog_md::render::render_doc;
use studyfrog_store::reviews::insert_review;
use studyfrog_store::schedule::save_mem_state;
use studyfrog_store::stackmap::STACK_FILE;

use crate::create::with_new_defaults;
use crate::create::{IdSource, check_title, meta, render_error, slug, unique_name, write_new};
use crate::rerate::replay_fsrs;
use crate::session::{ServiceError, VaultSession, current_settings};
use crate::settings::params_for;
use crate::xfer_model::{Bundle, ImportTarget, XItem, safe_folder};
use crate::xfer_plan::label;

pub(crate) fn target_from_opt(target: Option<&str>) -> ImportTarget {
    match target {
        None => ImportTarget::FromSource,
        Some(s) => ImportTarget::ExistingStack(s.to_owned()),
    }
}

pub(crate) fn new_stack_folder(
    session: &VaultSession,
    title: &str,
) -> Result<String, ServiceError> {
    let trimmed = title.trim();
    check_title(trimmed)?;
    let s = slug(trimmed);
    let base = if s.is_empty() { "stack" } else { s.as_str() };
    Ok(unique_name(base, "", |n| session.root.join(n).exists()))
}

fn combine_folder(base: &str, sub: &str) -> Result<String, String> {
    let folder = if sub.is_empty() {
        base.to_owned()
    } else {
        format!("{base}/{sub}")
    };
    if safe_folder(&folder) {
        Ok(folder)
    } else {
        Err(format!("\"{folder}\" is not a usable stack folder"))
    }
}

pub(crate) fn resolve_folder(
    item: &XItem,
    target: &ImportTarget,
    new_folder: Option<&str>,
) -> Result<String, String> {
    let sub = item.stack.trim();
    match target {
        ImportTarget::VaultRootNotes => Ok(String::new()),
        ImportTarget::NewStack(_) => {
            let base = new_folder.unwrap_or_default().trim();
            if base.is_empty() {
                return Err("choose a name for the new stack".to_owned());
            }
            combine_folder(base, sub)
        }
        ImportTarget::ExistingStack(t) => {
            let base = t.trim();
            if base.is_empty() {
                return Err("choose an existing stack".to_owned());
            }
            combine_folder(base, sub)
        }
        ImportTarget::FromSource => {
            if sub.is_empty() {
                return Err("it names no stack; choose a stack or Inbox to import into".to_owned());
            }
            combine_folder(sub, "")
        }
    }
}

/// Create a new top-level stack for `title` (slugged, unique) and return its folder name.
pub(crate) fn ensure_new_stack(
    session: &VaultSession,
    title: &str,
    ids: IdSource,
) -> Result<String, ServiceError> {
    let folder = new_stack_folder(session, title)?;
    let doc = Doc::Stack(StackDoc {
        meta: meta(title.trim(), &[], ids),
        description: String::new(),
        body: String::new(),
        extra: Vec::new(),
    });
    let doc = crate::item_keys::with_new_key(&session.conn, doc)?;
    let text = render_doc(&doc).map_err(render_error)?;
    write_new(&session.root.join(&folder).join(STACK_FILE), &text)?;
    Ok(folder)
}

pub(crate) fn insert_reviews(session: &VaultSession, uid: Uid, item: &XItem) {
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

pub(crate) fn doc_of(item: &XItem, uid: Uid, ids: IdSource, notes: &mut Vec<String>) -> Doc {
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
            (Some(w), false) => notes.push(format!("{title}: \"{w}\" is not a {key}")),
            _ => {}
        }
    }
    for (key, value) in &item.extra {
        doc = with_extra(&doc, key, Some(value));
    }
    if item.id.is_some() {
        doc
    } else {
        with_new_defaults(doc)
    }
}

fn doc_to_item(doc: &Doc, kind: ItemKind, text: String, answer: String) -> XItem {
    let meta = doc_meta(doc);
    let mut it = crate::xfer_model::item(kind, "", &text, &answer);
    (it.id, it.title, it.tags) = (Some(meta.uid), meta.title.clone(), meta.tags.clone());
    it.status = extra_value(doc, "status");
    it.priority = extra_value(doc, "priority");
    it.difficulty = extra_value(doc, "difficulty");
    let raw = match doc {
        Doc::Note(d) => &d.extra,
        Doc::Card(d) => &d.extra,
        Doc::Question(d) => &d.extra,
        Doc::Diary(d) => &d.extra,
        Doc::Stack(d) => &d.extra,
    };
    for entry in raw {
        if !matches!(entry.key.as_str(), "status" | "priority" | "difficulty")
            && let Some(val) = extra_raw(doc, &entry.key)
        {
            it.extra.push((entry.key.clone(), val));
        }
    }
    it
}

pub(crate) fn from_studyfrog_doc(file_name: &str, doc: &Doc) -> Result<Bundle, String> {
    let it = match doc {
        Doc::Stack(_) => {
            return Err(format!(
                "{file_name} is a StudyFrog stack file (_stack.md); import its folder instead"
            ));
        }
        Doc::Note(n) => doc_to_item(doc, ItemKind::Note, n.body.clone(), String::new()),
        Doc::Card(c) => doc_to_item(doc, ItemKind::Card, c.front.clone(), c.back.clone()),
        Doc::Question(q) => doc_to_item(
            doc,
            ItemKind::Question,
            q.prompt.clone(),
            q.reference.clone(),
        ),
        Doc::Diary(d) => doc_to_item(doc, ItemKind::Note, d.body.clone(), String::new()),
    };
    Ok(Bundle {
        items: vec![it],
        ..Bundle::default()
    })
}
