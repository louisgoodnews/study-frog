//! Document transformation helpers for in-rehearsal quick card editing.

use studyfrog_core::uid::Uid;
use studyfrog_md::doc::Doc;
use studyfrog_service::create_more::NewKind;
use studyfrog_service::document::LoadedDoc;

use crate::rehearsal_edit::EditCardModal;

/// Initialize edit modal state from a loaded document.
pub fn from_loaded(loaded: LoadedDoc, uid: Uid) -> EditCardModal {
    let (kind, title, text, answer, tags) = match &loaded.doc {
        Doc::Card(c) => (
            NewKind::Card,
            c.meta.title.clone(),
            c.front.clone(),
            c.back.clone(),
            c.meta.tags.join(", "),
        ),
        Doc::Question(q) => (
            NewKind::Question,
            q.meta.title.clone(),
            q.prompt.clone(),
            q.reference.clone(),
            q.meta.tags.join(", "),
        ),
        Doc::Note(n) => (
            NewKind::Note,
            n.meta.title.clone(),
            n.body.clone(),
            String::new(),
            n.meta.tags.join(", "),
        ),
        Doc::Stack(s) => (
            NewKind::Stack,
            s.meta.title.clone(),
            s.description.clone(),
            s.body.clone(),
            s.meta.tags.join(", "),
        ),
        Doc::Diary(d) => (
            NewKind::Note,
            d.meta.title.clone(),
            d.body.clone(),
            String::new(),
            d.meta.tags.join(", "),
        ),
    };
    EditCardModal {
        uid,
        path: loaded.path,
        hash: loaded.hash,
        doc: loaded.doc,
        kind,
        title,
        text,
        answer,
        tags,
        busy: false,
        error: None,
    }
}

/// Build a modified Doc with updated fields from the edit modal.
pub fn apply_to_doc(m: &EditCardModal) -> Doc {
    let tags: Vec<String> = m
        .tags
        .split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_owned)
        .collect();
    match &m.doc {
        Doc::Card(c) => {
            let mut c = c.clone();
            c.meta.title = m.title.clone();
            c.meta.tags = tags;
            c.front = m.text.clone();
            c.back = m.answer.clone();
            Doc::Card(c)
        }
        Doc::Question(q) => {
            let mut q = q.clone();
            q.meta.title = m.title.clone();
            q.meta.tags = tags;
            q.prompt = m.text.clone();
            q.reference = m.answer.clone();
            Doc::Question(q)
        }
        Doc::Note(n) => {
            let mut n = n.clone();
            n.meta.title = m.title.clone();
            n.meta.tags = tags;
            n.body = m.text.clone();
            Doc::Note(n)
        }
        other => other.clone(),
    }
}
