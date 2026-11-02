//! The editor: state and update logic. Pure.

use studyfrog_core::uid::Uid;
use studyfrog_md::blocks::{Block, parse_blocks};
use studyfrog_md::doc::Doc;
use studyfrog_service::document::{LoadedDoc, Saved};

use crate::edit_form::{EditForm, apply_form, form_from_doc};
use crate::effect::Effect;
use crate::msg::Msg;
use crate::state::{Notice, NoticeKind, Screen, State};

/// A document open for editing.
#[derive(Debug, Clone, PartialEq)]
pub struct Editor {
    /// The item.
    pub uid: Uid,
    /// Vault-relative path of the file.
    pub path: String,
    /// The document as last loaded or saved.
    pub doc: Doc,
    /// Fingerprint of the file as last loaded or saved.
    pub hash: u64,
    /// The form as last loaded or saved.
    pub original: EditForm,
    /// The form being edited.
    pub form: EditForm,
    /// Preview of each text area.
    pub previews: Vec<Vec<Block>>,
    /// The document being written, while a save is in flight.
    pub saving: Option<Doc>,
    /// Items whose text links to this one.
    pub backlinks: Vec<studyfrog_store::overview::ItemRow>,
    /// The familiarity over time, once read.
    pub curve: Option<Box<studyfrog_service::fam_curve::ItemCurve>>,
}

/// True when the form differs from what is on disk.
pub fn is_dirty(e: &Editor) -> bool {
    e.form != e.original
}

pub(crate) fn notice(
    state: State,
    kind: NoticeKind,
    text: impl Into<String>,
) -> (State, Vec<Effect>) {
    (
        State {
            notice: Some(Notice {
                kind,
                text: text.into(),
            }),
            ..state
        },
        Vec::new(),
    )
}

fn previews(form: &EditForm) -> Vec<Vec<Block>> {
    form.fields.iter().map(|f| parse_blocks(&f.text)).collect()
}

fn editor_new(loaded: LoadedDoc, uid: Uid) -> Editor {
    let form = form_from_doc(&loaded.doc);
    Editor {
        uid,
        path: loaded.path,
        doc: loaded.doc,
        hash: loaded.hash,
        previews: previews(&form),
        original: form.clone(),
        form,
        saving: None,
        backlinks: Vec::new(),
        curve: None,
    }
}

/// Unsaved edits that navigation or opening another item must not lose.
pub fn has_unsaved_changes(state: &State) -> bool {
    state.editor.as_ref().is_some_and(is_dirty)
}

/// Handle the editor's messages.
pub fn update(state: State, msg: Msg) -> (State, Vec<Effect>) {
    match msg {
        Msg::OpenItem(uid) => open_item(state, uid),
        Msg::DocumentLoaded(uid, result) => loaded(state, uid, result),
        Msg::FamCurveLoaded(uid, Ok(curve)) => crate::editor_edit::curve(state, uid, curve),
        Msg::BacklinksLoaded(uid, result) => crate::editor_edit::backlinks(state, uid, result),
        Msg::Edit(action) => crate::editor_edit::edit(state, action),
        Msg::SaveClicked => save(state),
        Msg::Saved(result) => saved(state, result),
        Msg::RevertClicked => revert(state),
        Msg::ReloadClicked => reload(state),
        Msg::CloseEditor => close(state),
        Msg::OpenLink(target) => (state, vec![Effect::ResolveLink(target)]),
        Msg::LinkResolved(target, result) => link_resolved(state, &target, result),
        _ => (state, Vec::new()),
    }
}

fn open_item(state: State, uid: Uid) -> (State, Vec<Effect>) {
    if has_unsaved_changes(&state) && state.editor.as_ref().is_some_and(|e| e.uid != uid) {
        return notice(
            state,
            NoticeKind::Error,
            "Save or revert your changes before opening another item.",
        );
    }
    (state, vec![Effect::LoadDocument(uid)])
}

fn loaded(state: State, uid: Uid, result: Result<LoadedDoc, String>) -> (State, Vec<Effect>) {
    match result {
        Ok(doc) => {
            let editor = editor_new(doc, uid);
            let ask = Effect::LoadBacklinks(uid, editor.form.title.clone());
            let state = State {
                editor: Some(editor),
                screen: Screen::Editor,
                notice: None,
                ..state
            };
            (state, vec![ask])
        }
        Err(e) => notice(state, NoticeKind::Error, format!("Could not open it: {e}")),
    }
}

fn save(state: State) -> (State, Vec<Effect>) {
    let Some(e) = &state.editor else {
        return (state, Vec::new());
    };
    if !is_dirty(e) || e.saving.is_some() {
        return (state, Vec::new());
    }
    match apply_form(&e.doc, &e.form) {
        Ok(doc) => {
            let effect = Effect::SaveDocument {
                path: e.path.clone(),
                doc: doc.clone(),
                hash: e.hash,
            };
            let mut state = state;
            if let Some(e) = &mut state.editor {
                e.saving = Some(doc);
            }
            (state, vec![effect])
        }
        Err(reason) => notice(state, NoticeKind::Error, reason),
    }
}

fn saved(state: State, result: Result<Saved, String>) -> (State, Vec<Effect>) {
    let mut state = state;
    let Some(e) = &mut state.editor else {
        return (state, Vec::new());
    };
    let pending = e.saving.take();
    match (result, pending) {
        (Ok(saved), Some(doc)) => {
            (e.doc, e.hash, e.original) = (doc, saved.hash, e.form.clone());
            state.vault = Some(saved.info);
            notice(state, NoticeKind::Info, "Saved.")
        }
        (Ok(_), None) => (state, Vec::new()),
        (Err(reason), _) => notice(state, NoticeKind::Error, format!("Not saved: {reason}")),
    }
}

fn revert(state: State) -> (State, Vec<Effect>) {
    let mut state = state;
    if let Some(e) = &mut state.editor {
        e.form = e.original.clone();
        e.previews = previews(&e.form);
    }
    (state, Vec::new())
}

fn reload(state: State) -> (State, Vec<Effect>) {
    match &state.editor {
        Some(e) => {
            let uid = e.uid;
            (state, vec![Effect::LoadDocument(uid)])
        }
        None => (state, Vec::new()),
    }
}

fn close(state: State) -> (State, Vec<Effect>) {
    if has_unsaved_changes(&state) {
        return notice(
            state,
            NoticeKind::Error,
            "You have unsaved changes: save, or revert them first.",
        );
    }
    (
        State {
            editor: None,
            screen: Screen::Library,
            ..state
        },
        Vec::new(),
    )
}

fn link_resolved(
    state: State,
    target: &str,
    result: Result<Option<Uid>, String>,
) -> (State, Vec<Effect>) {
    match result {
        Ok(Some(uid)) => open_item(state, uid),
        Ok(None) => notice(
            state,
            NoticeKind::Error,
            format!("No item is titled \"{target}\"."),
        ),
        Err(e) => notice(
            state,
            NoticeKind::Error,
            format!("Could not follow the link: {e}"),
        ),
    }
}
