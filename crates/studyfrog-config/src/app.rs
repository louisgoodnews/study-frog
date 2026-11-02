//! `app.toml`: per-user application settings, outside any vault.

use serde::Deserialize;
use studyfrog_core::settings::{
    AnswerEngine, AppSettings, MAX_READING_WPM as MAX_WPM, MIN_READING_WPM as MIN_WPM,
    ReadingTuning, answer_engine_from_name, answer_engine_name,
};

use crate::error::{ConfigError, config_error, from_toml, line_of_key};
use crate::values::toml_string;

/// File name inside the user's StudyFrog config directory.
pub const APP_CONFIG_FILE: &str = "app.toml";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawApp {
    vault: Option<String>,
    #[serde(default)]
    recent: Vec<String>,
    reading_wpm: Option<u32>,
    answer_engine: Option<String>,
    reading_fast_percent: Option<u32>,
    reading_slow_percent: Option<u32>,
    reading_min_words: Option<u32>,
    settings_tab: Option<String>,
    track_views: Option<bool>,
}

/// `value` if it is inside `range`, else an error naming the key and its line.
fn in_range(
    text: &str,
    key: &str,
    value: Option<u32>,
    range: std::ops::RangeInclusive<u32>,
) -> Result<Option<u32>, ConfigError> {
    match value {
        Some(v) if !range.contains(&v) => Err(config_error(
            line_of_key(text, key),
            format!(
                "{key} must be between {} and {}, not {v}",
                range.start(),
                range.end()
            ),
        )),
        other => Ok(other),
    }
}

/// Parse `app.toml`. Unknown keys are errors.
pub fn parse_app_settings(text: &str) -> Result<AppSettings, ConfigError> {
    let raw: RawApp = toml::from_str(text).map_err(|e| from_toml(text, &e))?;
    if let Some(wpm) = raw.reading_wpm.filter(|w| !(MIN_WPM..=MAX_WPM).contains(w)) {
        let line = line_of_key(text, "reading_wpm");
        return Err(config_error(
            line,
            format!("reading_wpm must be between {MIN_WPM} and {MAX_WPM}, not {wpm}"),
        ));
    }
    let answer_engine = match raw.answer_engine.as_deref() {
        None => AnswerEngine::default(),
        Some(name) => answer_engine_from_name(name).ok_or_else(|| {
            config_error(
                line_of_key(text, "answer_engine"),
                format!("answer_engine must be \"auto\" or \"words\", not \"{name}\""),
            )
        })?,
    };
    let reading = ReadingTuning {
        fast_percent: in_range(
            text,
            "reading_fast_percent",
            raw.reading_fast_percent,
            10..=100,
        )?,
        slow_percent: in_range(
            text,
            "reading_slow_percent",
            raw.reading_slow_percent,
            100..=1000,
        )?,
        min_words: in_range(text, "reading_min_words", raw.reading_min_words, 1..=200)?,
    };
    Ok(AppSettings {
        reading,
        vault: raw.vault,
        recent: raw.recent,
        reading_wpm: raw.reading_wpm,
        answer_engine,
        settings_tab: raw.settings_tab,
        views_untracked: raw.track_views == Some(false),
    })
}

/// Render settings as `app.toml`.
pub fn render_app_settings(s: &AppSettings) -> String {
    let mut out = String::from("# StudyFrog application settings (this machine and user).\n");
    if let Some(v) = &s.vault {
        out.push_str(&format!("vault = {}\n", toml_string(v)));
    }
    if !s.recent.is_empty() {
        let items: Vec<String> = s.recent.iter().map(|p| toml_string(p)).collect();
        out.push_str(&format!("recent = [{}]\n", items.join(", ")));
    }
    if s.answer_engine != AnswerEngine::default() {
        out.push_str(&format!(
            "answer_engine = \"{}\"\n",
            answer_engine_name(s.answer_engine)
        ));
    }
    for (key, value) in [
        ("reading_fast_percent", s.reading.fast_percent),
        ("reading_slow_percent", s.reading.slow_percent),
        ("reading_min_words", s.reading.min_words),
    ] {
        if let Some(v) = value {
            out.push_str(&format!("{key} = {v}\n"));
        }
    }
    if let Some(w) = s.reading_wpm {
        out.push_str(&format!("reading_wpm = {w}\n"));
    }
    if s.views_untracked {
        out.push_str("track_views = false\n");
    }
    if let Some(tab) = &s.settings_tab {
        out.push_str(&format!("settings_tab = {}\n", toml_string(tab)));
    }
    out
}
