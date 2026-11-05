//! Small pure helpers of the create dialog's form.

use studyfrog_core::civil::parse_date;
use studyfrog_service::create_more::NewKind;
use studyfrog_service::track::{DEFAULT_DUE_MINUTE, DueDate};

use crate::modal_types::CreateForm;
use crate::state::{Selection, State};

pub(crate) fn default_parent(state: &State) -> Option<String> {
    match &state.browser.selection {
        Selection::Stack(f) => Some(f.clone()),
        _ => state
            .vault
            .as_ref()
            .and_then(|v| v.overview.stacks.first().map(|s| s.folder.clone())),
    }
}

pub(crate) fn form_new(state: &State, kind: Option<NewKind>) -> CreateForm {
    let has_stacks = state
        .vault
        .as_ref()
        .is_some_and(|v| !v.overview.stacks.is_empty());
    CreateForm {
        kind: kind.unwrap_or(if has_stacks {
            NewKind::Card
        } else {
            NewKind::Stack
        }),
        title: String::new(),
        text: String::new(),
        answer: String::new(),
        tags: String::new(),
        parent: default_parent(state),
        also_in: None,
        another: false,
        error: None,
        busy: false,
        due: String::new(),
        reminders: vec![1440],
        priority: studyfrog_core::priority::Priority::Medium,
        difficulty: studyfrog_core::event::Difficulty::Difficult,
        confirm_discard: false,
        is_stack: true,
    }
}

pub(crate) fn toggled(list: &[u32], minutes: u32) -> Vec<u32> {
    let mut out: Vec<u32> = list.iter().copied().filter(|m| *m != minutes).collect();
    if out.len() == list.len() {
        out.push(minutes);
    }
    out.sort_unstable();
    out
}

pub(crate) fn tags_of(text: &str) -> Vec<String> {
    text.split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_owned)
        .collect()
}

pub(crate) fn due_of(form: &CreateForm) -> Option<DueDate> {
    let date = parse_date(form.due.trim())?;
    Some(DueDate {
        date,
        minute: DEFAULT_DUE_MINUTE,
        reminders: form.reminders.clone(),
    })
}

/// Why the form cannot be submitted yet, in words for the user.
pub fn problem(f: &CreateForm) -> Option<&'static str> {
    let blank = |s: &str| s.trim().is_empty();
    if !blank(&f.due) && parse_date(f.due.trim()).is_none() {
        return Some("The due date must look like 2026-12-31.");
    }
    match f.kind {
        NewKind::Stack | NewKind::Note if blank(&f.title) => Some("Give it a title."),
        NewKind::Card if blank(&f.text) || blank(&f.answer) => {
            Some("A card needs a front and a back.")
        }
        NewKind::Question if blank(&f.text) || blank(&f.answer) => {
            Some("A question needs a prompt and a reference answer.")
        }
        NewKind::Stack => None,
        _ if f.parent.is_none() => Some("Choose the stack it goes into."),
        _ => None,
    }
}
