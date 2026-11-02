//! Teachers and subjects: the vault's `subjects.toml`, and which subjects a stack or an item has.

use std::fs;

use studyfrog_config::error::describe_config_error;
use studyfrog_config::subjects::{
    SUBJECTS_FILE, Subject, SubjectBook, book_problem, parse_subjects, render_subjects,
    without_unknown_teachers,
};
use studyfrog_store::overview::StackRow;

use crate::create::write_new;
use crate::session::{ServiceError, VaultSession};

/// The vault's book, or an empty one with the reason when the file cannot be used.
pub fn load_subjects(session: &VaultSession) -> (SubjectBook, Option<String>) {
    match fs::read_to_string(session.root.join(SUBJECTS_FILE)) {
        Ok(text) => match parse_subjects(&text) {
            Ok(book) => (book, None),
            Err(e) => (
                SubjectBook::default(),
                Some(format!(
                    "{SUBJECTS_FILE} is not used: {}",
                    describe_config_error(&e)
                )),
            ),
        },
        Err(_) => (SubjectBook::default(), None),
    }
}

/// Check and store a book. A teacher that was removed leaves its subjects without a teacher.
pub fn save_subjects(
    session: &VaultSession,
    book: SubjectBook,
) -> Result<SubjectBook, ServiceError> {
    let book = without_unknown_teachers(&book);
    if let Some(why) = book_problem(&book) {
        return Err(ServiceError::Invalid(why));
    }
    write_new(&session.root.join(SUBJECTS_FILE), &render_subjects(&book))?;
    Ok(book)
}

/// The subjects a stack has: the ones it names, else those of the nearest stack above that names some.
pub fn stack_subjects(stacks: &[StackRow], folder: &str) -> Vec<String> {
    let mut at = stacks.iter().find(|s| s.folder == folder);
    while let Some(stack) = at {
        if !stack.subjects.is_empty() {
            return stack.subjects.clone();
        }
        at = stack
            .parent
            .as_deref()
            .and_then(|p| stacks.iter().find(|s| s.folder == p));
    }
    Vec::new()
}

/// Each subject with its teacher, for showing: "Mathematics (Ms Rivera)".
pub fn subjects_with_teachers(
    book: &SubjectBook,
    names: &[String],
) -> Vec<(String, Option<String>)> {
    names
        .iter()
        .map(|n| {
            let teacher = book
                .subjects
                .iter()
                .find(|s: &&Subject| s.name == *n)
                .and_then(|s| s.teacher.clone());
            (n.clone(), teacher)
        })
        .collect()
}
