//! Edits to the open document and the lookup of what links to it. Pure.

use studyfrog_core::uid::Uid;
use studyfrog_md::blocks::parse_blocks;

use crate::editor::notice;
use crate::editor_save::token_done;
use crate::effect::Effect;
use crate::msg::EditAction;
use crate::state::{NoticeKind, State};

/// Keep the curve of the open item.
pub(crate) fn curve(
    state: State,
    uid: Uid,
    curve: Box<studyfrog_service::fam_curve::ItemCurve>,
) -> (State, Vec<Effect>) {
    let mut state = state;
    if let Some(e) = state.editor.as_mut().filter(|e| e.uid == uid) {
        e.curve = Some(curve);
    }
    (state, Vec::new())
}

pub(crate) fn backlinks(
    state: State,
    uid: Uid,
    result: Result<Vec<studyfrog_store::overview::ItemRow>, String>,
) -> (State, Vec<Effect>) {
    let mut state = state;
    match (&mut state.editor, result) {
        (Some(e), Ok(rows)) if e.uid == uid => e.backlinks = rows,
        (_, Err(why)) => {
            return notice(
                state,
                NoticeKind::Error,
                format!("Could not find links: {why}"),
            );
        }
        _ => {}
    }
    (state, Vec::new())
}

pub(crate) fn edit(state: State, action: EditAction) -> (State, Vec<Effect>) {
    let mut state = state;
    let mut finished = true;
    if let Some(e) = &mut state.editor {
        match action {
            EditAction::Title(t) => {
                finished = token_done(&e.form.title, &t);
                e.form.title = t;
            }
            EditAction::Tags(t) => {
                finished = token_done(&e.form.tags, &t);
                e.form.tags = t;
            }
            EditAction::Priority(p) => e.form.priority = p,
            EditAction::Difficulty(d) => e.form.difficulty = d,
            EditAction::Status(s) => e.form.status = s,
            EditAction::Subject(name, on) => {
                e.form.subjects.retain(|s| *s != name);
                if on {
                    e.form.subjects.push(name);
                }
            }
            EditAction::AddReference(id) => {
                if !e.form.referenced_by.contains(&id) {
                    e.form.referenced_by.push(id);
                }
            }
            EditAction::RemoveReference(id) => {
                e.form.referenced_by.retain(|x| x != &id);
            }
            EditAction::AskDelete(on) => {
                e.confirm_delete = on;
                return (state, Vec::new());
            }
            EditAction::ConfirmDelete => {
                let uid = e.uid;
                state.editor = None;
                state.screen = crate::state::Screen::Library;
                let effect = Effect::Library(crate::library_bulk::LibraryEffect::Bulk(
                    vec![uid],
                    studyfrog_service::bulk::BulkEdit::Delete,
                ));
                return (state, vec![effect]);
            }
            EditAction::Field(i, text) => {
                if let Some(f) = e.form.fields.get_mut(i) {
                    finished = token_done(&f.text, &text);
                    f.text = text;
                    e.previews[i] = parse_blocks(&f.text);
                }
            }
        }
    }
    crate::editor_save::edited(state, finished)
}
