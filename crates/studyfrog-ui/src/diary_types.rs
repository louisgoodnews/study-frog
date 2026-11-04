//! Diary screen data types and state. Pure data and free functions.
//! See `docs/specs/diary.md`.

use std::collections::BTreeMap;

use studyfrog_core::civil::Date;
use studyfrog_core::diary::{RatingValue, STANDARD_SCALES};
use studyfrog_core::uid::Uid;
use studyfrog_store::diary::StoredDiaryEntry;

/// View mode for the diary screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiaryViewMode {
    /// Chronological list of entries.
    #[default]
    List,
    /// Calendar month grid view.
    MonthCalendar,
}

/// A user-defined custom rating scale configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomScaleDef {
    /// Alphanumeric identifier (e.g. "caffeine").
    pub id: String,
    /// Display name (e.g. "Caffeine intake").
    pub name: String,
    /// Explanatory prompt question.
    pub prompt: String,
    /// Minimum score bound (typically 1).
    pub min: i64,
    /// Maximum score bound (typically 5).
    pub max: i64,
    /// Optional descriptive labels for each level from min to max.
    pub labels: Vec<String>,
}

/// Settings governing the diary screen and prompt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiarySettings {
    /// Active standard scale IDs shown in the entry form.
    pub visible_scales: Vec<String>,
    /// User-defined custom rating scales.
    pub custom_scales: Vec<CustomScaleDef>,
    /// Whether the quiet "No diary entry yet today" hint shows on Home.
    pub show_home_hint: bool,
    /// Primary scale IDs for compact post-run prompt (up to 3).
    pub primary_scales: Vec<String>,
}

/// Draft entry being created or edited in the UI form.
#[derive(Debug, Clone, PartialEq)]
pub struct DiaryEditDraft {
    /// Stable ULID; `None` when creating a new entry.
    pub uid: Option<Uid>,
    /// Vault-relative path; `None` when creating a new entry.
    pub path: Option<String>,
    /// Civil date of the diary entry.
    pub date: Date,
    /// Optional custom title.
    pub title: String,
    /// Self-reported or tracked study duration in minutes.
    pub study_minutes: Option<u32>,
    /// Comma-separated tags string.
    pub tags: String,
    /// Ratings by scale ID.
    pub ratings: BTreeMap<String, RatingValue>,
    /// Comma-separated stack references.
    pub stacks: String,
    /// Referenced item ULIDs.
    pub items: Vec<Uid>,
    /// Associated rehearsal run ULIDs.
    pub runs: Vec<Uid>,
    /// Referenced exam / session IDs.
    pub exams: Vec<String>,
    /// Markdown body text for reflections.
    pub content: String,
    /// Whether any field has changed since opening.
    pub is_dirty: bool,
}

/// State of the Diary screen.
#[derive(Debug, Clone, PartialEq)]
pub struct DiaryState {
    /// Selected date in calendar or list.
    pub selected_date: Date,
    /// Current view mode (list vs month calendar).
    pub view_mode: DiaryViewMode,
    /// Optional stack filter.
    pub filter_stack: Option<String>,
    /// Rating scale to filter by.
    pub filter_rating_scale: Option<String>,
    /// Minimum rating threshold (1..=5).
    pub filter_min_rating: Option<u8>,
    /// Search query text.
    pub search_query: String,
    /// Currently open entry draft in the editor form.
    pub active_entry: Option<DiaryEditDraft>,
    /// Loaded diary entries from the vault index.
    pub entries: Vec<StoredDiaryEntry>,
    /// Whether the user dismissed today's Home hint in this session.
    pub hint_dismissed: bool,
    /// Diary configuration settings.
    pub settings: DiarySettings,
    /// Draft for creating a new custom scale in Settings.
    pub custom_scale_draft: Option<CustomScaleDef>,
}

/// Side effects requested by diary actions.
#[derive(Debug, Clone, PartialEq)]
pub enum DiaryEffect {
    /// Load all diary entries from the SQLite index.
    LoadEntries,
    /// Save a diary entry to markdown and reindex.
    SaveEntry(Box<DiaryEditDraft>),
    /// Delete a diary entry file and reindex.
    DeleteEntry(Uid),
}

/// Default diary settings.
pub fn diary_settings_default() -> DiarySettings {
    DiarySettings {
        visible_scales: STANDARD_SCALES.iter().map(|&s| s.to_owned()).collect(),
        custom_scales: Vec::new(),
        show_home_hint: true,
        primary_scales: vec![
            "focus".to_owned(),
            "energy".to_owned(),
            "satisfaction".to_owned(),
        ],
    }
}

impl Default for DiarySettings {
    fn default() -> Self {
        diary_settings_default()
    }
}

/// Default state for the diary module.
pub fn diary_state_default() -> DiaryState {
    DiaryState {
        selected_date: Date {
            year: 2026,
            month: 11,
            day: 4,
        },
        view_mode: DiaryViewMode::List,
        filter_stack: None,
        filter_rating_scale: None,
        filter_min_rating: None,
        search_query: String::new(),
        active_entry: None,
        entries: Vec::new(),
        hint_dismissed: false,
        settings: diary_settings_default(),
        custom_scale_draft: None,
    }
}

impl Default for DiaryState {
    fn default() -> Self {
        diary_state_default()
    }
}

/// Create a new blank draft for `date`.
pub fn new_draft(date: Date) -> DiaryEditDraft {
    DiaryEditDraft {
        uid: None,
        path: None,
        date,
        title: String::new(),
        study_minutes: None,
        tags: String::new(),
        ratings: BTreeMap::new(),
        stacks: String::new(),
        items: Vec::new(),
        runs: Vec::new(),
        exams: Vec::new(),
        content: String::new(),
        is_dirty: false,
    }
}

/// Create a draft from an existing stored entry.
pub fn draft_from_entry(e: &StoredDiaryEntry) -> DiaryEditDraft {
    DiaryEditDraft {
        uid: Some(e.uid),
        path: Some(e.path.clone()),
        date: e.date,
        title: e.title.clone(),
        study_minutes: e.study_minutes,
        tags: e.tags.join(", "),
        ratings: e.ratings.clone(),
        stacks: e.stacks.join(", "),
        items: e.items.clone(),
        runs: e.runs.clone(),
        exams: e.exams.clone(),
        content: e.content.clone(),
        is_dirty: false,
    }
}
