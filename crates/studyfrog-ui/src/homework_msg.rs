//! Messages, state and effects of the homework screen (STORY-0167).

use studyfrog_calendar::homework::HomeworkItem;
use studyfrog_calendar::homework_query::HomeworkTab;
use studyfrog_core::civil::LocalTime;
use studyfrog_core::uid::Uid;

use crate::homework_form::{HomeworkEdit, HomeworkForm};

/// Everything the homework screen can be asked to do.
#[derive(Debug, Clone, PartialEq)]
pub enum HomeworkMsg {
    /// Show a status tab.
    Tab(HomeworkTab),
    /// Open the dialog for a new item.
    New,
    /// Open the dialog for an existing item.
    EditItem(Uid),
    /// Close the dialog without saving.
    Close,
    /// A field of the open dialog changed.
    Edit(HomeworkEdit),
    /// Save the open dialog.
    Save,
    /// Move an item on to its next status.
    Advance(Uid),
    /// Tick or untick a checklist step of an item.
    ToggleStep(Uid, usize),
    /// The dialog's save finished.
    Saved(HomeworkResult),
    /// An action other than saving finished.
    Changed(HomeworkResult),
}

/// What the homework screen shows besides the list itself.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct HomeworkUi {
    /// The status tab in front.
    pub tab: HomeworkTab,
    /// The create or edit dialog, when open.
    pub form: Option<HomeworkForm>,
    /// Why the last action was refused.
    pub error: Option<String>,
}

/// The items read back after a change, with the clock they were read at.
pub type HomeworkResult = Result<(Vec<HomeworkItem>, LocalTime), String>;

/// Work the homework screen asks the worker to do. Each one answers with the items as stored.
#[derive(Debug, Clone, PartialEq)]
pub enum HomeworkEffect {
    /// Store the item. A new one (`existing` is `None`) gets a fresh id.
    Save {
        /// The item being edited, if it exists.
        existing: Option<Uid>,
        /// The item as the form describes it.
        item: HomeworkItem,
    },
    /// Move an item on: not started, in progress, completed, then reopened.
    Advance(Uid),
    /// Tick or untick one checklist step.
    ToggleStep(Uid, usize),
}
