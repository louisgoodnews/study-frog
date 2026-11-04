//! Messages for diary screen events, user actions, and background results.

use studyfrog_core::civil::Date;
use studyfrog_core::diary::RatingValue;
use studyfrog_core::uid::Uid;
use studyfrog_store::diary::StoredDiaryEntry;

use crate::diary_types::{CustomScaleDef, DiaryViewMode};

/// Diary-related messages handled by the update loop.
#[derive(Debug, Clone, PartialEq)]
pub enum DiaryMsg {
    /// Select a civil date in calendar or list.
    SelectDate(Date),
    /// Navigate month backwards (-1) or forwards (+1).
    StepMonth(i32),
    /// Switch between List and MonthCalendar view mode.
    SetViewMode(DiaryViewMode),
    /// Set or clear stack filter.
    SetFilterStack(Option<String>),
    /// Set or clear scale and minimum rating filter.
    SetFilterRating(Option<String>, Option<u8>),
    /// Search input edited.
    SetSearch(String),
    /// Start writing a new diary entry for `date`.
    NewEntry(Date),
    /// Open an existing entry for editing by its UID.
    EditEntry(Uid),
    /// Cancel editing and discard the active draft.
    CloseEditor,
    /// Title text edited in draft form.
    UpdateDraftTitle(String),
    /// Study minutes edited in draft form.
    UpdateDraftMinutes(Option<u32>),
    /// Rating updated or cleared for a scale ID.
    UpdateDraftRating(String, Option<RatingValue>),
    /// Stacks text edited in draft form.
    UpdateDraftStacks(String),
    /// Tags text edited in draft form.
    UpdateDraftTags(String),
    /// Markdown body text edited in draft form.
    UpdateDraftContent(String),
    /// Save the active draft.
    SaveDraft,
    /// Delete an entry by its UID.
    DeleteEntry(Uid),
    /// Entries loaded from the store.
    EntriesLoaded(Vec<StoredDiaryEntry>),
    /// Dismiss the Home screen quiet nudge for today.
    DismissHomeHint,
    /// Enable or disable visibility of a standard scale in Settings.
    ToggleScaleVisibility(String, bool),
    /// Add a new user-defined custom scale in Settings.
    AddCustomScale(CustomScaleDef),
    /// Remove a custom scale by ID in Settings.
    RemoveCustomScale(String),
    /// Toggle whether the Home screen hint is enabled.
    SetShowHomeHint(bool),
    /// Update draft for new custom scale creation.
    UpdateCustomDraft(Option<CustomScaleDef>),
    /// Update rating for a scale in the post-rehearsal diary prompt.
    PromptRating(String, Option<RatingValue>),
    /// Update reflection note in the post-rehearsal diary prompt.
    PromptNote(String),
    /// Submit the post-rehearsal diary prompt.
    PromptSubmit,
    /// Skip and dismiss the post-rehearsal diary prompt.
    PromptSkip,
    /// Expand the post-rehearsal prompt into the full diary editor.
    PromptExpand,
    /// Notification message from diary operations.
    Saved(Result<String, String>),
}
