//! Settings as plain data. They are read from TOML files (`studyfrog.toml` at the vault root and
//! `_stack.toml` in a stack folder) by `studyfrog-config`, never from Markdown front-matter.

use crate::scheduler::Scheduler;

/// Per-stack settings from `_stack.toml`. Every field is optional: `None` means "use the vault default".
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct StackSettings {
    /// Scheduling algorithm.
    pub scheduler: Option<Scheduler>,
    /// FSRS target retention in `(0, 1)`.
    pub request_retention: Option<f32>,
    /// New items introduced per day.
    pub daily_new_limit: Option<u32>,
    /// Reviews per day.
    pub daily_review_limit: Option<u32>,
    /// Whether familiarity tempers this stack's intervals; `None` follows the vault.
    pub familiarity: Option<bool>,
}

/// Vault-wide settings from `studyfrog.toml`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VaultSettings {
    /// Default scheduler for stacks that do not choose one.
    pub scheduler: Scheduler,
    /// Default FSRS target retention in `(0, 1)`.
    pub request_retention: f32,
    /// Upper bound for any interval, in days.
    pub maximum_interval_days: u32,
    /// Custom FSRS weights; `None` means the published defaults.
    pub weights: Option<[f32; 21]>,
    /// How strongly familiarity tempers intervals (0 switches it off; `docs/research/familiarity-and-fsrs.md`).
    pub familiarity_strength: f32,
    /// A due item at or above this familiarity (0 to 1) is left out of the daily queue; 1 leaves nothing out.
    pub familiarity_threshold: f32,
}

/// Defaults used when `studyfrog.toml` is absent or leaves a key out.
pub fn vault_settings_default() -> VaultSettings {
    VaultSettings {
        scheduler: Scheduler::Fsrs,
        request_retention: 0.9,
        maximum_interval_days: 36500,
        weights: None,
        familiarity_strength: 2.0,
        familiarity_threshold: 1.0,
    }
}

/// Most recent vaults remembered by the app.
pub const MAX_RECENT_VAULTS: usize = 8;

/// Slowest accepted reading speed, in words per minute.
pub const MIN_READING_WPM: u32 = 50;
/// Fastest accepted reading speed, in words per minute.
pub const MAX_READING_WPM: u32 = 1000;

/// How typed answers are compared with the reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AnswerEngine {
    /// The language model when it is installed, else word matching.
    #[default]
    Auto,
    /// Word matching only; the language model is never loaded.
    Words,
}

/// Persisted name of an answer engine choice.
pub fn answer_engine_name(e: AnswerEngine) -> &'static str {
    match e {
        AnswerEngine::Auto => "auto",
        AnswerEngine::Words => "words",
    }
}

/// Parse an answer engine choice.
pub fn answer_engine_from_name(name: &str) -> Option<AnswerEngine> {
    match name {
        "auto" => Some(AnswerEngine::Auto),
        "words" => Some(AnswerEngine::Words),
        _ => None,
    }
}

/// The user's overrides of the reading-speed thresholds (see the rehearsal spec); `None` keeps the
/// built-in value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ReadingTuning {
    /// Reading in less than this percentage of the expected time counts as fast (10 to 100).
    pub fast_percent: Option<u32>,
    /// Reading in more than this percentage of the expected time counts as slow (100 to 1000).
    pub slow_percent: Option<u32>,
    /// Answers shorter than this many words say too little about familiarity (1 to 200).
    pub min_words: Option<u32>,
}

/// Per-user application settings (not part of any vault): where the app last worked.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AppSettings {
    /// The vault opened last; the app reopens it at start-up.
    pub vault: Option<String>,
    /// Recently opened vaults, newest first, without duplicates.
    pub recent: Vec<String>,
    /// Measured reading speed in words per minute; `None` means the built-in default is used.
    pub reading_wpm: Option<u32>,
    /// How typed answers are compared.
    pub answer_engine: AnswerEngine,
    /// Thresholds for the reading-speed suggestions.
    pub reading: ReadingTuning,
    /// The tab of the settings screen shown last (a stable key such as `rehearsal`).
    pub settings_tab: Option<String>,
    /// Do not record how long items are viewed (the default is to record it).
    pub views_untracked: bool,
    /// The columns of the library list, as stored words (`title,type,status`); `None` uses the defaults.
    pub library_columns: Option<String>,
    /// Do not show the reviews the schedule plans as entries of the calendar (the default is to show them).
    pub calendar_reviews_off: bool,
    /// The tab each stack was last left on, as `folder:tab` (at most [`MAX_STACK_TABS`]).
    pub stack_tabs: Vec<String>,
}

/// Most stacks whose tab is remembered.
pub const MAX_STACK_TABS: usize = 200;

/// `settings` after opening `path`: it becomes the current vault and moves to the front of the recent list.
pub fn with_opened_vault(settings: &AppSettings, path: &str) -> AppSettings {
    let mut recent: Vec<String> = std::iter::once(path.to_owned())
        .chain(
            settings
                .recent
                .iter()
                .filter(|p| p.as_str() != path)
                .cloned(),
        )
        .collect();
    recent.truncate(MAX_RECENT_VAULTS);
    AppSettings {
        vault: Some(path.to_owned()),
        recent,
        ..settings.clone()
    }
}
