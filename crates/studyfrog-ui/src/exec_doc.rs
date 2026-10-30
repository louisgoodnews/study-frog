//! Performing the document and graph effects: reading, saving and resolving links, and the link graph.

use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::doc::Doc;
use studyfrog_service::document::{load_document, resolve_link, save_document};

use crate::exec::Worker;
use crate::msg::Msg;

/// Read a document for editing.
pub fn load_document_msg(w: &Worker, uid: Uid) -> Msg {
    match &w.session {
        Some(s) => Msg::DocumentLoaded(uid, load_document(s, uid).map_err(|e| e.to_string())),
        None => Msg::DocumentLoaded(uid, Err("no vault is open".to_owned())),
    }
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
