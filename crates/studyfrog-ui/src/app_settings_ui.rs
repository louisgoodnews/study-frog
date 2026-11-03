//! The tabs of the settings screen and the remembered tab, kept in `app.toml` (one file per profile). Pure.

use studyfrog_core::settings::AppSettings;

use crate::effect::Effect;
use crate::state::{Notice, NoticeKind, State};

/// A tab of the settings screen, one per domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    /// Vault-wide and application-wide basics.
    General,
    /// Theme and layout.
    Appearance,
    /// Scheduler, retention and intervals.
    Study,
    /// Reading speed and run behaviour.
    Rehearsal,
    /// Answer comparison and the assistant.
    Answers,
    /// Notification behaviour.
    Notifications,
    /// Backups, restore and the vault's files.
    Data,
    /// Teachers and subjects.
    Subjects,
}

/// The tabs that have settings today, in order. A tab joins this list when its first setting arrives.
pub const SHOWN: [SettingsTab; 6] = [
    SettingsTab::Study,
    SettingsTab::Appearance,
    SettingsTab::Rehearsal,
    SettingsTab::Answers,
    SettingsTab::Data,
    SettingsTab::Subjects,
];

const KEYS: [(SettingsTab, &str, &str); 8] = [
    (SettingsTab::General, "general", "General"),
    (SettingsTab::Appearance, "appearance", "Appearance"),
    (SettingsTab::Study, "study", "Study and scheduling"),
    (SettingsTab::Rehearsal, "rehearsal", "Rehearsal"),
    (SettingsTab::Answers, "answers", "Answers and AI"),
    (SettingsTab::Notifications, "notifications", "Notifications"),
    (SettingsTab::Data, "data", "Data"),
    (SettingsTab::Subjects, "subjects", "Subjects and teachers"),
];

fn entry(tab: SettingsTab) -> (&'static str, &'static str) {
    KEYS.iter()
        .find(|(t, _, _)| *t == tab)
        .map_or(("", ""), |(_, key, label)| (key, label))
}

/// The stable key written to `app.toml`.
pub fn tab_key(tab: SettingsTab) -> &'static str {
    entry(tab).0
}

/// The name shown on the tab.
pub fn tab_label(tab: SettingsTab) -> &'static str {
    entry(tab).1
}

/// The tab for a stored key.
pub fn tab_from_key(key: &str) -> Option<SettingsTab> {
    KEYS.iter().find(|(_, k, _)| *k == key).map(|(t, _, _)| *t)
}

/// The tab to show: the remembered one if it has settings, else the first.
pub fn current_tab(app: &AppSettings) -> SettingsTab {
    app.settings_tab
        .as_deref()
        .and_then(tab_from_key)
        .filter(|t| SHOWN.contains(t))
        .unwrap_or(SHOWN[0])
}

/// Messages of the tab bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTabMsg {
    /// A tab was chosen.
    Select(SettingsTab),
    /// Move to the next (1) or previous (-1) tab, wrapping around.
    Step(i32),
}

fn select(state: State, tab: SettingsTab) -> (State, Vec<Effect>) {
    if current_tab(&state.app) == tab {
        return (state, Vec::new());
    }
    let app = AppSettings {
        settings_tab: Some(tab_key(tab).to_owned()),
        ..state.app.clone()
    };
    let effects = vec![Effect::SaveAppSettings(app.clone())];
    (State { app, ..state }, effects)
}

/// Handle a tab message.
pub fn update(state: State, msg: SettingsTabMsg) -> (State, Vec<Effect>) {
    match msg {
        SettingsTabMsg::Select(tab) => select(state, tab),
        SettingsTabMsg::Step(by) => {
            let at = SHOWN.iter().position(|t| *t == current_tab(&state.app));
            let n = SHOWN.len() as i32;
            let next = (at.unwrap_or(0) as i32 + by).rem_euclid(n) as usize;
            select(state, SHOWN[next])
        }
    }
}

/// Switch the recording of item views on or off.
pub fn set_track_views(state: State, on: bool) -> (State, Vec<Effect>) {
    let app = AppSettings {
        views_untracked: !on,
        ..state.app.clone()
    };
    let effects = vec![Effect::SaveAppSettings(app.clone())];
    (State { app, ..state }, effects)
}

/// The result of storing `app.toml`: a failure is reported, success changes nothing.
pub fn saved(state: State, result: Result<(), String>) -> (State, Vec<Effect>) {
    match result {
        Ok(()) => (state, Vec::new()),
        Err(e) => {
            let notice = Some(Notice {
                kind: NoticeKind::Error,
                text: format!("Could not remember the vault: {e}"),
            });
            (State { notice, ..state }, Vec::new())
        }
    }
}
