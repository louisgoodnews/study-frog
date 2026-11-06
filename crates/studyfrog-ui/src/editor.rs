//! The editor: state and update logic. Pure.

use studyfrog_core::uid::Uid;
use studyfrog_md::blocks::{Block, parse_blocks};
use studyfrog_md::doc::Doc;
use studyfrog_service::document::LoadedDoc;

use crate::edit_form::{EditForm, form_from_doc};
use crate::effect::Effect;
use crate::msg::Msg;
use crate::state::{Notice, NoticeKind, Screen, State};

/// A document open for editing.
pub type EditorState = Editor;

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
    /// The document and form being written, while a save is in flight.
    pub saving: Option<(Doc, EditForm)>,
    /// When the first edit not yet saved was made (Unix milliseconds).
    pub dirty_since: Option<i64>,
    /// Edits were made while a save ran: save again when it is done.
    pub again: bool,
    /// Why the last save failed.
    pub fail: Option<String>,
    /// The file changed outside StudyFrog; nothing is saved until the user chooses.
    pub conflict: bool,
    /// How many times the document was saved during this visit.
    pub saves: u32,
    /// "Keep mine" was chosen after a conflict: the next load keeps the edits.
    pub keep_mine: bool,
    /// What to do once everything is saved (leave, open another item, exit).
    pub after: Option<Box<Msg>>,
    /// Items whose text links to this one.
    pub backlinks: Vec<studyfrog_store::overview::ItemRow>,
    /// The familiarity over time, once read.
    pub curve: Option<Box<studyfrog_service::fam_curve::ItemCurve>>,
    /// Whether the inline delete confirmation bar is shown.
    pub confirm_delete: bool,
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
        dirty_since: None,
        again: false,
        fail: None,
        conflict: false,
        saves: 0,
        after: None,
        keep_mine: false,
        backlinks: Vec::new(),
        curve: None,
        confirm_delete: false,
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
        Msg::SaveClicked => crate::editor_save::save(state),
        Msg::Saved(result) => crate::editor_save::saved(state, result),
        Msg::KeepMineClicked => keep_mine(state),
        Msg::RevertClicked => revert(state),
        Msg::ReloadClicked => reload(state),
        Msg::CloseEditor => close(state),
        Msg::OpenLink(target) => (state, vec![Effect::ResolveLink(target)]),
        Msg::LinkResolved(target, result) => link_resolved(state, &target, result),
        _ => (state, Vec::new()),
    }
}

fn open_item(state: State, uid: Uid) -> (State, Vec<Effect>) {
    let state = if state.editor.as_ref().is_some_and(|e| e.uid != uid) {
        let (state, waiting) = crate::editor_save::flush_before(state, Msg::OpenItem(uid));
        if let Some(effects) = waiting {
            return (state, effects);
        }
        state
    } else {
        state
    };
    (state, vec![Effect::LoadDocument(uid)])
}

/// "Keep mine" after a conflict: read the file again for its new fingerprint and save the edits over it.
fn keep_mine(state: State) -> (State, Vec<Effect>) {
    let Some(e) = state.editor.as_ref().filter(|e| e.conflict) else {
        return (state, Vec::new());
    };
    let uid = e.uid;
    let mut state = state;
    if let Some(e) = &mut state.editor {
        e.keep_mine = true;
    }
    (state, vec![Effect::LoadDocument(uid)])
}

fn loaded(state: State, uid: Uid, result: Result<LoadedDoc, String>) -> (State, Vec<Effect>) {
    match result {
        Ok(doc) => {
            let mut editor = editor_new(doc, uid);
            let mine = state
                .editor
                .as_ref()
                .filter(|e| e.uid == uid && e.keep_mine)
                .map(|e| e.form.clone());
            let kept = mine.is_some();
            if let Some(form) = mine {
                editor.previews = previews(&form);
                (editor.form, editor.dirty_since) = (form, Some(state.wall_ms));
            }
            let ask = Effect::LoadBacklinks(uid, editor.form.title.clone());
            let state = State {
                editor: Some(editor),
                screen: Screen::Editor,
                notice: None,
                ..state
            };
            if !kept {
                return (state, vec![ask]);
            }
            let (state, mut effects) = crate::editor_save::save(state);
            effects.push(ask);
            (state, effects)
        }
        Err(e) => notice(state, NoticeKind::Error, format!("Could not open it: {e}")),
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
    let (state, waiting) = crate::editor_save::flush_before(state, Msg::CloseEditor);
    if let Some(effects) = waiting {
        return (state, effects);
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
