//! Adding a picture to a text area: choose the file, copy it into the vault, put the Markdown for it at
//! the end of the text. Pure.

use crate::effect::Effect;
use crate::images::{append_line, image_markdown};
use crate::modal::{CreateForm, Field, Modal};
use crate::state::{Notice, NoticeKind, State};

/// The text area a picture is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageTarget {
    /// A text area of the open item, by index.
    Item(usize),
    /// A text area of the Create dialog.
    Create(Field),
}

/// What a finished "add a picture" gives back: its vault path, or nothing when the user cancelled.
pub type Added = Result<Option<String>, String>;

/// The user asked to add a picture.
pub fn pick(state: State, target: ImageTarget) -> (State, Vec<Effect>) {
    (state, vec![Effect::AddImage(target)])
}

fn insert_into(text: &str, rel: &str) -> String {
    append_line(text, &image_markdown(rel))
}

fn into_create(form: CreateForm, field: Field, rel: &str) -> CreateForm {
    match field {
        Field::Text => CreateForm {
            text: insert_into(&form.text, rel),
            ..form
        },
        Field::Answer => CreateForm {
            answer: insert_into(&form.answer, rel),
            ..form
        },
        _ => form,
    }
}

/// The picture was copied (or the user cancelled, or it failed).
pub fn added(state: State, target: ImageTarget, result: Added) -> (State, Vec<Effect>) {
    let rel = match result {
        Ok(Some(rel)) => rel,
        Ok(None) => return (state, Vec::new()),
        Err(e) => {
            let notice = Some(Notice {
                kind: NoticeKind::Error,
                text: format!("Could not add the picture: {e}"),
            });
            return (State { notice, ..state }, Vec::new());
        }
    };
    let mut state = state;
    match target {
        ImageTarget::Item(i) => {
            if let Some(f) = state.editor.as_mut().and_then(|e| e.form.fields.get_mut(i)) {
                f.text = insert_into(&f.text, &rel);
            }
        }
        ImageTarget::Create(field) => {
            if let Some(Modal::Create(form)) = state.modal.take() {
                state.modal = Some(Modal::Create(into_create(form, field, &rel)));
            }
        }
    }
    (state, Vec::new())
}
