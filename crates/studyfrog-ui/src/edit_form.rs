//! The editor's form: the editable parts of a document as text, and the way back to a document.
//! Everything else in the file (unknown front-matter keys, ids, timestamps) is carried over by
//! cloning the original document, so editing can never drop it. Pure.

use studyfrog_core::event::{Difficulty, difficulty_from_name, difficulty_name};
use studyfrog_core::priority::{Priority, priority_from_name, priority_name};
use studyfrog_core::status::Status;
use studyfrog_md::doc::Doc;
use studyfrog_md::extra::{extra_value, with_extra};
use studyfrog_md::names::{extra_names, with_extra_names};
use studyfrog_md::refs::{references_of, with_references};
use studyfrog_md::status::{status_of, with_status};

/// One multi-line text area.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditField {
    /// Label shown above the area (also its accessible name).
    pub label: &'static str,
    /// The text.
    pub text: String,
    /// Visible rows.
    pub rows: usize,
}

/// Title, tags and the kind-specific text areas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditForm {
    /// Title.
    pub title: String,
    /// Tags, comma separated.
    pub tags: String,
    /// Text areas in display order.
    pub fields: Vec<EditField>,
    /// The priority the item states itself; `None`: inherit.
    pub priority: Option<Priority>,
    /// The difficulty the item states itself; `None`: inherit.
    pub difficulty: Option<Difficulty>,
    /// The status the item states itself; `None`: derived from its history.
    pub status: Option<Status>,
    /// The subjects a stack names (empty for other kinds and for a stack that names none).
    pub subjects: Vec<String>,
    /// The other stacks that reference this item (by stack UID string).
    pub referenced_by: Vec<String>,
}

/// A change in the editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditAction {
    /// The title field.
    Title(String),
    /// The tags field.
    Tags(String),
    /// A text area, by index.
    Field(usize, String),
    /// The item's own priority; `None` takes the stack's.
    Priority(Option<Priority>),
    /// The item's own difficulty; `None` takes the stack's.
    Difficulty(Option<Difficulty>),
    /// The item's own status; `None` leaves it to the history.
    Status(Option<Status>),
    /// A subject of a stack on or off.
    Subject(String, bool),
    /// Add a reference to this stack (by stack UID string).
    AddReference(String),
    /// Remove a reference to this stack (by stack UID string).
    RemoveReference(String),
}

fn field(label: &'static str, text: &str, rows: usize) -> EditField {
    EditField {
        label,
        text: text.to_owned(),
        rows,
    }
}

/// The form for a document.
pub fn form_from_doc(doc: &Doc) -> EditForm {
    let (meta, fields) = match doc {
        Doc::Note(d) => (&d.meta, vec![field("Body", &d.body, 16)]),
        Doc::Card(d) => (
            &d.meta,
            vec![field("Front", &d.front, 6), field("Back", &d.back, 6)],
        ),
        Doc::Question(d) => (
            &d.meta,
            vec![
                field("Prompt", &d.prompt, 6),
                field("Reference answer", &d.reference, 6),
            ],
        ),
        Doc::Stack(d) => (
            &d.meta,
            vec![
                field("Description", &d.description, 2),
                field("Body", &d.body, 10),
            ],
        ),
        Doc::Diary(d) => (&d.meta, vec![field("Body", &d.body, 16)]),
    };
    EditForm {
        title: meta.title.clone(),
        tags: meta.tags.join(", "),
        fields,
        priority: extra_value(doc, "priority").and_then(|w| priority_from_name(&w)),
        difficulty: extra_value(doc, "difficulty").and_then(|w| difficulty_from_name(&w)),
        status: status_of(doc),
        subjects: extra_names(doc, "subjects"),
        referenced_by: references_of(doc),
    }
}

fn single_line(what: &str, text: &str) -> Result<(), String> {
    if text
        .chars()
        .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}'))
    {
        return Err(format!(
            "{what} must be a single line without control characters"
        ));
    }
    Ok(())
}

fn tags_of(text: &str) -> Result<Vec<String>, String> {
    let tags: Vec<String> = text
        .split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_owned)
        .collect();
    tags.iter().try_for_each(|t| single_line("a tag", t))?;
    Ok(tags)
}

fn text(form: &EditForm, i: usize) -> String {
    form.fields
        .get(i)
        .map(|f| f.text.clone())
        .unwrap_or_default()
}

/// The document with the form's values applied; everything the form does not show is kept.
pub fn apply_form(doc: &Doc, form: &EditForm) -> Result<Doc, String> {
    if form.title.trim().is_empty() {
        return Err("the title must not be empty".to_owned());
    }
    single_line("the title", &form.title)?;
    let tags = tags_of(&form.tags)?;
    let mut out = doc.clone();
    match &mut out {
        Doc::Note(d) => {
            (d.meta.title, d.meta.tags, d.body) = (form.title.clone(), tags, text(form, 0))
        }
        Doc::Card(d) => {
            (d.meta.title, d.meta.tags) = (form.title.clone(), tags);
            (d.front, d.back) = (text(form, 0), text(form, 1));
        }
        Doc::Question(d) => {
            (d.meta.title, d.meta.tags) = (form.title.clone(), tags);
            (d.prompt, d.reference) = (text(form, 0), text(form, 1));
        }
        Doc::Stack(d) => {
            (d.meta.title, d.meta.tags) = (form.title.clone(), tags);
            single_line("the description", &text(form, 0))?;
            (d.description, d.body) = (text(form, 0), text(form, 1));
        }
        Doc::Diary(d) => {
            (d.meta.title, d.meta.tags, d.body) = (form.title.clone(), tags, text(form, 0));
        }
    }
    let out = with_extra(&out, "priority", form.priority.map(priority_name));
    let out = with_extra(&out, "difficulty", form.difficulty.map(difficulty_name));
    let out = with_references(&out, &form.referenced_by);
    let out = if form.status == status_of(doc) {
        out
    } else {
        with_status(&out, form.status)
    };
    if !matches!(out, Doc::Stack(_)) {
        return Ok(out);
    }
    Ok(with_extra_names(&out, "subjects", &form.subjects))
}
