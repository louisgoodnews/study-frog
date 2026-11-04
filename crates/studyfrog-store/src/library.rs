//! Building the in-memory [`Library`] from a scan.

use std::collections::HashMap;

use studyfrog_core::arena::arena_push;
use studyfrog_core::idx::Idx;
use studyfrog_core::items::{Card, Note, Question};
use studyfrog_core::library::{Library, library_new};
use studyfrog_core::meta::Meta;
use studyfrog_core::settings::StackSettings;
use studyfrog_core::stack::Stack;
use studyfrog_core::uid::Uid;
use studyfrog_md::doc::{CardDoc, Doc, NoteDoc, QuestionDoc, StackDoc};

use crate::scan::Scan;

/// Title of the implicit stack for items outside every stack.
pub const INBOX_TITLE: &str = "Inbox";

fn inbox() -> Stack {
    Stack {
        meta: Meta {
            uid: Uid(0),
            title: INBOX_TITLE.to_owned(),
            tags: Vec::new(),
            aliases: Vec::new(),
            created: None,
            updated: None,
        },
        description: "Items outside any stack".to_owned(),
        settings: StackSettings::default(),
    }
}

/// A core stack from its `_stack.md` and (optional) settings.
fn stack_from(doc: &StackDoc, settings: Option<&StackSettings>) -> Stack {
    Stack {
        meta: doc.meta.clone(),
        description: doc.description.clone(),
        settings: settings.copied().unwrap_or_default(),
    }
}

type StackIndex = HashMap<String, Idx<Stack>>;

fn push_stacks(scan: &Scan, lib: &mut Library) -> StackIndex {
    let mut index = StackIndex::new();
    for f in &scan.files {
        if let (Some(folder), Ok(Doc::Stack(doc))) = (&f.stack, &f.outcome)
            && let Ok(idx) = arena_push(
                &mut lib.stacks,
                stack_from(doc, scan.stack_settings.get(folder)),
            )
        {
            index.insert(folder.clone(), idx);
        }
    }
    index
}

fn stack_of(folder: &Option<String>, index: &StackIndex, lib: &mut Library) -> Option<Idx<Stack>> {
    let found = folder.as_ref().and_then(|f| index.get(f)).copied();
    found.or_else(|| inbox_idx(lib))
}

/// The inbox stack, created on first use (always the stack with uid 0).
fn inbox_idx(lib: &mut Library) -> Option<Idx<Stack>> {
    let existing = lib.stacks.items.iter().position(|s| s.meta.uid == Uid(0));
    match existing {
        Some(i) => u32::try_from(i).ok().map(studyfrog_core::idx::idx),
        None => arena_push(&mut lib.stacks, inbox()).ok(),
    }
}

fn push_note(lib: &mut Library, d: &NoteDoc, stack: Idx<Stack>) {
    let note = Note {
        meta: d.meta.clone(),
        stack,
        rehearse_title: d.rehearse_title,
        body: d.body.clone(),
    };
    let _ = arena_push(&mut lib.notes, note);
}

fn push_card(lib: &mut Library, d: &CardDoc, stack: Idx<Stack>) {
    let card = Card {
        meta: d.meta.clone(),
        stack,
        reversible: d.reversible,
        front: d.front.clone(),
        back: d.back.clone(),
    };
    let _ = arena_push(&mut lib.cards, card);
}

fn push_question(lib: &mut Library, d: &QuestionDoc, stack: Idx<Stack>) {
    let question = Question {
        meta: d.meta.clone(),
        stack,
        prompt: d.prompt.clone(),
        reference: d.reference.clone(),
        accepted: d.accepted.clone(),
        rubric: d.rubric.clone(),
        min_score: d.min_score,
    };
    let _ = arena_push(&mut lib.questions, question);
}

/// Build the library: stacks first, then every note, card and question in path order.
/// Files that failed to parse are left out (they are reported by the scan).
pub fn build_library(scan: &Scan) -> Library {
    let mut lib = library_new();
    let index = push_stacks(scan, &mut lib);
    for f in &scan.files {
        let Ok(doc) = &f.outcome else { continue };
        let Some(stack) = stack_of(&f.stack, &index, &mut lib) else {
            continue;
        };
        match doc {
            Doc::Stack(_) => {}
            Doc::Note(d) => push_note(&mut lib, d, stack),
            Doc::Card(d) => push_card(&mut lib, d, stack),
            Doc::Question(d) => push_question(&mut lib, d, stack),
            Doc::Diary(_) => {}
        }
    }
    lib
}
