//! The saved-session types of the Classroom browser, as plain data (TASK-0295). Re-exported from
//! `classroom_types`, so the screen and the worker keep their import paths.

use studyfrog_core::classroom::SourceRef;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;

/// One saved session as the browser lists it (TASK-0295).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedSession {
    /// Stable identity.
    pub id: Uid,
    /// The session title.
    pub title: String,
    /// When the session was created.
    pub created_at: UnixMs,
    /// What the lecture was grounded in; shown as the topic.
    pub sources: Vec<SourceRef>,
    /// Chat turns so far.
    pub turns: usize,
    /// Tasks set so far.
    pub tasks: usize,
}

/// A message about the saved sessions: the browser's actions and the list read back.
#[derive(Debug, Clone, PartialEq)]
pub enum SavedMsg {
    /// Load this saved session onto the screen, where it left off.
    Resume(Uid),
    /// Write this session's transcript to a Markdown file in the vault.
    Export(Uid),
    /// Ask for confirmation before this session is deleted.
    AskDelete(Uid),
    /// Keep the session whose delete was asked for.
    CancelDelete,
    /// Delete the session whose delete was asked for.
    ConfirmDelete,
    /// The saved sessions as stored now, and what the last action reported.
    Listed {
        /// Every saved session, newest first.
        sessions: Vec<SavedSession>,
        /// What the action did, e.g. where an export was written.
        notice: Option<String>,
        /// What went wrong, if anything.
        error: Option<String>,
    },
}

/// Work on the saved sessions, run by the worker against the store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SavedEffect {
    /// Read the saved sessions back.
    List,
    /// Load one saved session, as stored now.
    Load {
        /// The session.
        id: Uid,
    },
    /// Write the transcript of one session as Markdown.
    Export {
        /// The session.
        id: Uid,
    },
    /// Delete one session and its rows.
    Delete {
        /// The session.
        id: Uid,
    },
}
