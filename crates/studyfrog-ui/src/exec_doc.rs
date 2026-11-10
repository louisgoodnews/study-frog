//! Performing the document and graph effects: reading, saving and resolving links, and the link graph.

use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::doc::Doc;
use studyfrog_service::document::{load_document, resolve_link, save_document};

use studyfrog_service::session::poll_vault;

use crate::exec::Worker;
use crate::msg::Msg;
use crate::subjects_ui::SubjectsMsg;

/// Read a document for editing.
pub fn load_document_msg(w: &Worker, uid: Uid) -> Msg {
    match &w.session {
        Some(s) => Msg::DocumentLoaded(uid, load_document(s, uid).map_err(|e| e.to_string())),
        None => Msg::DocumentLoaded(uid, Err("no vault is open".to_owned())),
    }
}

/// Read a document for editing during rehearsal.
pub fn load_rehearsal_doc_msg(w: &Worker, uid: Uid) -> Msg {
    let res = match &w.session {
        Some(s) => load_document(s, uid).map_err(|e| e.to_string()),
        None => Err("no vault is open".to_owned()),
    };
    Msg::RehearsalEdit(crate::rehearsal_edit::RehearsalEditMsg::Loaded(
        uid,
        Box::new(res),
    ))
}

/// Write an edited document from rehearsal.
pub fn save_rehearsal_doc_msg(
    w: &mut Worker,
    uid: Uid,
    path: &str,
    doc: &Doc,
    hash: u64,
    now_ns: i64,
) -> Msg {
    let res = match &mut w.session {
        Some(s) => save_document(s, path, doc, hash, UnixMs(now_ns / 1_000_000))
            .map(|_| uid)
            .map_err(|e| e.to_string()),
        None => Err("no vault is open".to_owned()),
    };
    Msg::RehearsalEdit(crate::rehearsal_edit::RehearsalEditMsg::Saved(res))
}

/// Append a comment to an item during rehearsal.
pub fn append_rehearsal_comment_msg(
    w: &mut Worker,
    uid: Uid,
    comment: studyfrog_core::comment::ItemComment,
    now_ns: i64,
) -> Msg {
    let res = match &mut w.session {
        Some(s) => {
            studyfrog_service::document::append_comment(s, uid, comment, UnixMs(now_ns / 1_000_000))
                .map(|_| uid)
                .map_err(|e| e.to_string())
        }
        None => Err("no vault is open".to_owned()),
    };
    Msg::RehearsalEdit(crate::rehearsal_edit::RehearsalEditMsg::Saved(res))
}

/// Write an edited document if the file is unchanged since it was loaded.
pub fn save_document_msg(w: &mut Worker, path: &str, doc: &Doc, hash: u64, now_ns: i64) -> Msg {
    let Some(s) = &mut w.session else {
        return Msg::Saved(Err("no vault is open".to_owned()));
    };
    Msg::Saved(
        save_document(s, path, doc, hash, UnixMs(now_ns / 1_000_000)).map_err(|e| e.to_string()),
    )
}

/// Find the item a wikilink points to.
pub fn resolve_link_msg(w: &Worker, target: String) -> Msg {
    let result = match &w.session {
        Some(s) => resolve_link(s, &target).map_err(|e| e.to_string()),
        None => Err("no vault is open".to_owned()),
    };
    Msg::LinkResolved(target, result)
}

/// Read the link graph from the index.
pub fn load_graph_msg(w: &Worker) -> Msg {
    match &w.session {
        Some(s) => Msg::GraphLoaded(
            studyfrog_store::overview::load_link_graph(&s.conn).map_err(|e| e.to_string()),
        ),
        None => Msg::GraphLoaded(Err("no vault is open".to_owned())),
    }
}

/// The items that link to the one with this title.
pub fn backlinks(w: &Worker, uid: Uid, title: &str) -> Msg {
    let result = match &w.session {
        Some(session) => studyfrog_store::overview::linking_to(&session.conn, title, uid)
            .map_err(|e| e.to_string()),
        None => Ok(Vec::new()),
    };
    Msg::BacklinksLoaded(uid, result)
}

/// The links to an item and its familiarity over time.
pub fn item_extras(w: &Worker, uid: Uid, title: &str, now_ns: i64) -> Vec<Msg> {
    let now = studyfrog_core::time::UnixMs(now_ns / 1_000_000);
    let curve = match &w.session {
        Some(s) => studyfrog_service::fam_curve::item_curve(&s.conn, uid, now)
            .map(Box::new)
            .map_err(|e| e.to_string()),
        None => Err("no vault is open".to_owned()),
    };
    vec![backlinks(w, uid, title), Msg::FamCurveLoaded(uid, curve)]
}

/// Read the subjects book, or store a changed one and read it back.
pub fn subjects(w: &Worker, change: Option<studyfrog_config::subjects::SubjectBook>) -> Msg {
    use studyfrog_service::subjects::{load_subjects, save_subjects};
    let Some(session) = &w.session else {
        return Msg::Subjects(SubjectsMsg::Loaded(Ok(Default::default())));
    };
    let result = match change {
        Some(book) => save_subjects(session, book)
            .map(|b| (b, None))
            .map_err(|e| format!("Could not save the subjects: {e}")),
        None => Ok(load_subjects(session)),
    };
    Msg::Subjects(SubjectsMsg::Loaded(result))
}

/// Check the open vault for changed files.
pub fn poll(w: &mut Worker, now_ns: i64) -> Msg {
    match &mut w.session {
        Some(session) => Msg::VaultPolled(poll_vault(session, now_ns).map_err(|e| e.to_string())),
        None => Msg::VaultPolled(Ok(None)),
    }
}
