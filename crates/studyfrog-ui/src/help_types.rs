//! The help screen's state, messages and effects, as plain data.

use studyfrog_md::blocks::Block;
use studyfrog_service::help::{Hit, PageRef, UserPage};

use crate::state::Screen;

/// A user page being written or changed.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PageEdit {
    /// The name the page had when editing began (`None`: a new page).
    pub original: Option<String>,
    /// The file name, as typed.
    pub name: String,
    /// The Markdown, as typed.
    pub text: String,
    /// Why the last save failed, shown next to the form.
    pub problem: Option<String>,
    /// A save is running.
    pub saving: bool,
}

/// What the help screen shows.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct HelpState {
    /// The page showing (`None`: the first page).
    pub page: Option<PageRef>,
    /// The page's Markdown, parsed once when the page opens.
    pub blocks: Vec<Block>,
    /// Pages to go back to, newest last.
    pub back: Vec<PageRef>,
    /// The screen help was opened from, for "Back to ...".
    pub came_from: Option<Screen>,
    /// The search text.
    pub search: String,
    /// What the search found.
    pub hits: Vec<Hit>,
    /// The user's pages, as last read.
    pub user: Vec<UserPage>,
    /// The page being written.
    pub edit: Option<PageEdit>,
    /// The question before deleting the open user page is showing.
    pub ask_delete: bool,
    /// The tutorial under way.
    pub tutorial: Option<crate::tutorial::Tutorial>,
}

/// Something that happened on the help screen.
#[derive(Debug, Clone, PartialEq)]
pub enum HelpMsg {
    /// Open help at the page of the screen that is showing.
    ForScreen,
    /// Open a page.
    Open(PageRef),
    /// A link in a page was clicked.
    Link(String),
    /// Go to the page shown before.
    Back,
    /// Return to the screen help was opened from.
    Leave,
    /// The search text changed.
    Search(String),
    /// The user's pages were read.
    Loaded(Result<Vec<UserPage>, String>),
    /// Start a new user page.
    New,
    /// Start a user page as a copy of the open page.
    Copy,
    /// Change the open user page.
    Edit,
    /// The name field changed.
    EditName(String),
    /// The text changed.
    EditText(String),
    /// Store the page being written.
    Save,
    /// Stop writing without storing.
    Cancel,
    /// The page was stored (or not), with the pages after it.
    Saved(Result<Vec<UserPage>, String>),
    /// Show or hide the question before deleting.
    AskDelete(bool),
    /// Delete the open user page.
    Delete,
    /// The page was deleted (or not).
    Deleted(Result<Vec<UserPage>, String>),
    /// Something about the tutorial.
    Tutorial(crate::tutorial::TutorialMsg),
}

/// What the help screen asks the worker to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelpEffect {
    /// Read the user's pages.
    Load,
    /// Write a user page, renaming it from `old` when the name changed.
    Save {
        /// The name before editing (`None`: a new page).
        old: Option<String>,
        /// The name.
        name: String,
        /// The Markdown.
        text: String,
    },
    /// Delete a user page.
    Delete(String),
}
