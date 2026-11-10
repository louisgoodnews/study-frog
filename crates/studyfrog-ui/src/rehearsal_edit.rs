//! In-rehearsal quick card/item editing.
//! Pauses the run, loads document front-matter, and updates item content on save.

use studyfrog_core::uid::Uid;
use studyfrog_md::doc::Doc;
use studyfrog_rehearsal::input::Action;
use studyfrog_service::create_more::NewKind;
use studyfrog_service::document::LoadedDoc;

use crate::effect::Effect;
use crate::modal::Modal;
use crate::state::{Notice, NoticeKind, State};

/// Data of the rehearsal edit modal dialog.
#[derive(Debug, Clone, PartialEq)]
pub struct EditCardModal {
    /// Item identifier.
    pub uid: Uid,
    /// Absolute or vault-relative path to source file.
    pub path: String,
    /// Content hash when loaded to detect external changes.
    pub hash: u64,
    /// Original loaded document.
    pub doc: Doc,
    /// Item kind (card, question, note).
    pub kind: NewKind,
    /// Item title.
    pub title: String,
    /// Front / prompt / body markdown text.
    pub text: String,
    /// Back / reference answer markdown text.
    pub answer: String,
    /// Comma-separated tag list.
    pub tags: String,
    /// Whether an asynchronous save operation is in flight.
    pub busy: bool,
    /// Error message from load or save, if any.
    pub error: Option<String>,
}

/// Messages of the in-rehearsal editor.
#[derive(Debug, Clone, PartialEq)]
pub enum RehearsalEditMsg {
    /// Request opening modal for the item.
    Open(Uid),
    /// Loaded document from disk.
    Loaded(Uid, Result<LoadedDoc, String>),
    /// Title changed.
    EditTitle(String),
    /// Front/prompt body changed.
    EditText(String),
    /// Back/answer body changed.
    EditAnswer(String),
    /// Tags string changed.
    EditTags(String),
    /// Trigger save operation.
    Save,
    /// Save completed.
    Saved(Result<Uid, String>),
    /// Close the modal without saving.
    Close,
}

/// Side effects performed for in-rehearsal editing.
#[derive(Debug, Clone, PartialEq)]
pub enum RehearsalEditEffect {
    /// Read a document for editing.
    Load(Uid),
    /// Save edited document: (uid, path, doc, hash).
    Save(Uid, String, Doc, u64),
}

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
    let tags: Vec<String> = m.tags.split(',').map(str::trim).filter(|t| !t.is_empty()).map(str::to_owned).collect();
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

/// Handle messages for in-rehearsal quick card editing.
pub fn update(state: State, msg: RehearsalEditMsg) -> (State, Vec<Effect>) {
    match msg {
        RehearsalEditMsg::Open(uid) => open_edit(state, uid),
        RehearsalEditMsg::Loaded(uid, res) => doc_loaded(state, uid, res),
        RehearsalEditMsg::EditTitle(t) => with_modal(state, |mut m| { m.title = t; m }),
        RehearsalEditMsg::EditText(t) => with_modal(state, |mut m| { m.text = t; m }),
        RehearsalEditMsg::EditAnswer(a) => with_modal(state, |mut m| { m.answer = a; m }),
        RehearsalEditMsg::EditTags(t) => with_modal(state, |mut m| { m.tags = t; m }),
        RehearsalEditMsg::Save => save_edit(state),
        RehearsalEditMsg::Saved(res) => edit_saved(state, res),
        RehearsalEditMsg::Close => close_edit(state),
    }
}

fn with_modal(mut state: State, f: impl FnOnce(EditCardModal) -> EditCardModal) -> (State, Vec<Effect>) {
    if let Some(Modal::EditItem(m)) = state.modal.take() {
        state.modal = Some(Modal::EditItem(Box::new(f(*m))));
    }
    (state, Vec::new())
}

fn open_edit(state: State, uid: Uid) -> (State, Vec<Effect>) {
    let running = state.rehearse.run.as_ref().is_some_and(|r| {
        r.session.run.state == studyfrog_core::run::RunState::Running
    });
    let (state, mut effects) = if running {
        crate::run_update::act(state, Action::Pause)
    } else {
        (state, Vec::new())
    };
    effects.push(Effect::RehearsalEdit(RehearsalEditEffect::Load(uid)));
    (state, effects)
}

fn doc_loaded(mut state: State, uid: Uid, res: Result<LoadedDoc, String>) -> (State, Vec<Effect>) {
    match res {
        Ok(loaded) => {
            state.modal = Some(Modal::EditItem(Box::new(from_loaded(loaded, uid))));
            (state, Vec::new())
        }
        Err(e) => {
            state.notice = Some(Notice {
                kind: NoticeKind::Error,
                text: format!("Could not load card for editing: {e}"),
            });
            (state, Vec::new())
        }
    }
}

fn save_edit(mut state: State) -> (State, Vec<Effect>) {
    let Some(Modal::EditItem(ref mut m)) = state.modal else { return (state, Vec::new()); };
    if m.busy { return (state, Vec::new()); }
    if m.title.trim().is_empty() {
        m.error = Some("title cannot be empty".to_owned());
        return (state, Vec::new());
    }
    m.busy = true;
    m.error = None;
    let doc = apply_to_doc(m);
    let effect = Effect::RehearsalEdit(RehearsalEditEffect::Save(m.uid, m.path.clone(), doc, m.hash));
    (state, vec![effect])
}

fn edit_saved(mut state: State, res: Result<Uid, String>) -> (State, Vec<Effect>) {
    match res {
        Ok(uid) => {
            state.modal = None;
            state.notice = Some(Notice { kind: NoticeKind::Info, text: "Item updated".to_owned() });
            (state, vec![Effect::LoadItem(uid)])
        }
        Err(e) => {
            if let Some(Modal::EditItem(ref mut m)) = state.modal {
                m.busy = false;
                m.error = Some(e);
            }
            (state, Vec::new())
        }
    }
}

fn close_edit(mut state: State) -> (State, Vec<Effect>) {
    state.modal = None;
    (state, Vec::new())
}
