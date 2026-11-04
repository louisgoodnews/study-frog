//! Planning the transition from a stack to a normal folder (ADR 0011, TASK-0217).

use std::fs;

use studyfrog_core::uid::Uid;
use studyfrog_md::doc::Doc;
use studyfrog_md::goals::goals_of;
use studyfrog_store::items::{item_path, load_indexed_items};
use studyfrog_store::overview::load_overview;
use studyfrog_store::stackmap::STACK_FILE;

use crate::session::{ServiceError, VaultSession};

/// What will change when a stack becomes a normal folder.
#[derive(Debug, Clone, PartialEq)]
pub struct UnmarkPlan {
    /// The folder of the stack.
    pub folder: String,
    /// Title of the stack.
    pub title: String,
    /// Identity of the stack.
    pub uid: Uid,
    /// Whether `_stack.toml` exists in this folder.
    pub has_settings: bool,
    /// Status stated in `_stack.md`.
    pub status: Option<studyfrog_core::status::Status>,
    /// Number of goals under `## Goals`.
    pub goals_count: usize,
    /// Subjects named by the stack.
    pub subjects: Vec<String>,
    /// Sub-stacks inside this folder that remain stacks.
    pub sub_stacks: Vec<String>,
    /// Items in the vault that reference this stack (uid and path).
    pub referencing_items: Vec<(Uid, String)>,
    /// Items in this stack that will belong to the stack above.
    pub items_count: usize,
    /// Target stack folder above, or `None` for the Inbox.
    pub target_folder: Option<String>,
    /// Target stack title, e.g. "French" or "Inbox".
    pub target_title: String,
}

fn count_goals(body: &str) -> usize {
    goals_of(body)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .count()
}

/// Plan what will change when unmarking `folder`, without touching any files.
pub fn plan_unmark(session: &VaultSession, folder: &str) -> Result<UnmarkPlan, ServiceError> {
    let stacks = load_overview(&session.conn)
        .map_err(ServiceError::Store)?
        .stacks;
    let stack = stacks
        .iter()
        .find(|s| s.folder == folder)
        .ok_or_else(|| ServiceError::Invalid(format!("no stack at \"{folder}\"")))?;

    let target_folder = stack.parent.clone();
    let target_title = target_folder.as_ref().map_or_else(
        || "Inbox".to_owned(),
        |p| {
            stacks
                .iter()
                .find(|s| s.folder == *p)
                .map_or_else(|| "Inbox".to_owned(), |s| s.title.clone())
        },
    );

    let prefix = format!("{folder}/");
    let sub_stacks: Vec<String> = stacks
        .iter()
        .filter(|s| s.folder.starts_with(&prefix))
        .map(|s| s.folder.clone())
        .collect();

    let indexed = load_indexed_items(&session.conn).map_err(ServiceError::Store)?;
    let items_count = indexed.iter().filter(|i| i.stack == stack.uid).count();

    let mut referencing_items = Vec::new();
    for item in indexed.iter().filter(|i| i.also_in.contains(&stack.uid)) {
        if let Some(path) = item_path(&session.conn, item.uid).map_err(ServiceError::Store)? {
            referencing_items.push((item.uid, path));
        }
    }

    let stack_file = session.root.join(folder).join(STACK_FILE);
    let goals_count = fs::read_to_string(&stack_file)
        .ok()
        .and_then(|t| studyfrog_md::parse::parse_doc(&t).ok())
        .map_or(0, |d| match d {
            Doc::Stack(s) => count_goals(&s.body),
            _ => 0,
        });

    let has_settings = session.root.join(folder).join("_stack.toml").is_file();

    Ok(UnmarkPlan {
        folder: folder.to_owned(),
        title: stack.title.clone(),
        uid: stack.uid,
        has_settings,
        status: stack.status,
        goals_count,
        subjects: stack.subjects.clone(),
        sub_stacks,
        referencing_items,
        items_count,
        target_folder,
        target_title,
    })
}
