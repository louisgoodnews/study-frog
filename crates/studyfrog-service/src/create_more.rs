//! Creating any kind of item in one place: stacks (also inside other stacks), notes, cards and
//! questions. Every item except a stack lives in a stack; a stack may live in another stack.

use studyfrog_core::items::DEFAULT_MIN_SCORE;
use studyfrog_md::doc::{CardDoc, Doc, NoteDoc, QuestionDoc, StackDoc};
use studyfrog_md::extra::with_extra;
use studyfrog_md::render::render_doc;
use studyfrog_store::stackmap::STACK_FILE;

use crate::create::{
    IdSource, check_title, default_card_title, invalid, meta, refresh, render_error, slug,
    unique_name, with_new_defaults, write_new,
};
use crate::session::{ServiceError, VaultInfo, VaultSession};
use studyfrog_core::event::difficulty_name;
use studyfrog_core::priority::priority_name;

/// What can be created.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewKind {
    /// A stack (a folder with a `_stack.md`).
    Stack,
    /// A note.
    Note,
    /// A flashcard.
    Card,
    /// A question with a reference answer.
    Question,
}

/// The input of [`create_item`].
#[derive(Debug, Clone, Copy)]
pub struct NewItem<'a> {
    /// What to create.
    pub kind: NewKind,
    /// Title; empty means "derive it from the text" for cards and questions.
    pub title: &'a str,
    /// The main text: a stack's description, a note's body, a card's front, a question's prompt.
    pub text: &'a str,
    /// The second text: a card's back or a question's reference answer.
    pub answer: &'a str,
    /// Tags.
    pub tags: &'a [String],
    /// The stack (vault-relative folder) it goes into. Required for everything but a stack, where
    /// `None` means a top-level stack.
    pub parent: Option<&'a str>,
    /// Where the text came from, such as `docs/book.pdf#page=3` (written as the `source` key).
    pub source: Option<&'a str>,
    /// The priority the user chose; `None` leaves the default (medium; a stack states none).
    pub priority: Option<studyfrog_core::priority::Priority>,
    /// The difficulty the user chose; `None` leaves the default (difficult; a stack states none).
    pub difficulty: Option<studyfrog_core::event::Difficulty>,
}

/// What was created.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Created {
    /// The title it got.
    pub title: String,
    /// Its vault-relative path.
    pub path: String,
}

fn parent_dir(session: &VaultSession, folder: &str) -> Result<std::path::PathBuf, ServiceError> {
    let dir = session.root.join(folder);
    if folder.is_empty() || !dir.join(STACK_FILE).is_file() {
        return Err(invalid("that folder is not a stack"));
    }
    Ok(dir)
}

fn doc_of(item: &NewItem<'_>, title: &str, ids: IdSource) -> Result<Doc, ServiceError> {
    let m = meta(title, item.tags, ids);
    Ok(match item.kind {
        NewKind::Stack => Doc::Stack(StackDoc {
            meta: m,
            description: item.text.trim_end().to_owned(),
            body: String::new(),
            extra: Vec::new(),
        }),
        NewKind::Note => Doc::Note(NoteDoc {
            meta: m,
            rehearse_title: false,
            body: item.text.trim_end().to_owned(),
            extra: Vec::new(),
        }),
        NewKind::Card => Doc::Card(CardDoc {
            meta: m,
            reversible: false,
            front: item.text.trim_end().to_owned(),
            back: item.answer.trim_end().to_owned(),
            extra: Vec::new(),
        }),
        NewKind::Question => Doc::Question(QuestionDoc {
            meta: m,
            prompt: item.text.trim_end().to_owned(),
            reference: item.answer.trim_end().to_owned(),
            accepted: Vec::new(),
            rubric: Vec::new(),
            min_score: DEFAULT_MIN_SCORE,
            extra: Vec::new(),
        }),
    })
}

/// The title to use, or why the input is not enough.
fn title_of(item: &NewItem<'_>) -> Result<String, ServiceError> {
    let given = item.title.trim();
    match item.kind {
        NewKind::Stack | NewKind::Note if given.is_empty() => Err(invalid("give it a title")),
        NewKind::Card | NewKind::Question
            if item.text.trim().is_empty() || item.answer.trim().is_empty() =>
        {
            let (first, second) = if item.kind == NewKind::Card {
                ("front", "back")
            } else {
                ("prompt", "reference answer")
            };
            Err(invalid(&format!("it needs both a {first} and a {second}")))
        }
        _ if given.is_empty() => Ok(default_card_title(item.text)),
        _ => Ok(given.to_owned()),
    }
}

/// The document with the priority and difficulty the user chose.
fn with_choices(doc: Doc, item: &NewItem<'_>) -> Doc {
    let doc = match item.priority {
        Some(p) => with_extra(&doc, "priority", Some(priority_name(p))),
        None => doc,
    };
    match item.difficulty {
        Some(d) => with_extra(&doc, "difficulty", Some(difficulty_name(d))),
        None => doc,
    }
}

/// Create the item as a new file (or folder) and reindex once.
pub fn create_item(
    session: &mut VaultSession,
    item: &NewItem<'_>,
    ids: IdSource,
) -> Result<(VaultInfo, Created), ServiceError> {
    let title = title_of(item)?;
    check_title(&title)?;
    if item.source.is_some_and(|s| s.contains(['"', '\n'])) {
        return Err(invalid(
            "the source of an item cannot contain quotes or line breaks",
        ));
    }
    if item.kind != NewKind::Stack && item.parent.is_none() {
        return Err(invalid("choose the stack it goes into"));
    }
    let base = match item.parent {
        Some(folder) => parent_dir(session, folder)?,
        None => session.root.clone(),
    };
    let doc = with_extra(
        &with_choices(with_new_defaults(doc_of(item, &title, ids)?), item),
        "source",
        item.source.map(|s| format!("\"{s}\"")).as_deref(),
    );
    let text = render_doc(&doc).map_err(render_error)?;
    let rel_of = |name: &str| match item.parent {
        Some(folder) => format!("{folder}/{name}"),
        None => name.to_owned(),
    };
    let path = if item.kind == NewKind::Stack {
        let folder = unique_name(&slug(&title), "", |n| base.join(n).exists());
        write_new(&base.join(&folder).join(STACK_FILE), &text)?;
        format!("{}/{STACK_FILE}", rel_of(&folder))
    } else {
        let name = unique_name(&slug(&title), ".md", |n| base.join(n).exists());
        write_new(&base.join(&name), &text)?;
        rel_of(&name)
    };
    Ok((refresh(session)?, Created { title, path }))
}
