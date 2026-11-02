//! Edits to the open document and the lookup of what links to it. Pure.

use studyfrog_core::uid::Uid;
use studyfrog_md::blocks::parse_blocks;

use crate::editor::notice;
use crate::effect::Effect;
use crate::msg::EditAction;
use crate::state::{NoticeKind, State};

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
    if let Some(e) = &mut state.editor {
        match action {
            EditAction::Title(t) => e.form.title = t,
            EditAction::Tags(t) => e.form.tags = t,
            EditAction::Priority(p) => e.form.priority = p,
            EditAction::Difficulty(d) => e.form.difficulty = d,
            EditAction::Status(s) => e.form.status = s,
            EditAction::Subject(name, on) => {
                e.form.subjects.retain(|s| *s != name);
                if on {
                    e.form.subjects.push(name);
                }
            }
            EditAction::Field(i, text) => {
                if let Some(f) = e.form.fields.get_mut(i) {
                    f.text = text;
                    e.previews[i] = parse_blocks(&f.text);
                }
            }
        }
    }
    (state, Vec::new())
}
