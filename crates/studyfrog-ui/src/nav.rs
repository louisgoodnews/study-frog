//! Changing screens: which ones may be left, and what each one needs loaded.

use crate::effect::Effect;
use crate::state::{Screen, State};

/// Go to `screen` if allowed, asking for whatever data it shows.
pub fn navigate(state: State, screen: Screen) -> (State, Vec<Effect>) {
    let allowed = (screen == Screen::Welcome || state.vault.is_some())
        && (screen == Screen::Editor || !crate::editor::has_unsaved_changes(&state));
    if allowed && screen == Screen::Graph {
        (State { screen, ..state }, vec![Effect::LoadGraph])
    } else if allowed && screen == Screen::Calendar {
        (State { screen, ..state }, vec![Effect::LoadSessions])
    } else if allowed && (screen == Screen::Home || screen == Screen::Library) {
        (
            State { screen, ..state },
            vec![Effect::LoadHome, Effect::LoadSessions],
        )
    } else if allowed && screen == Screen::Agent {
        (State { screen, ..state }, vec![Effect::LoadAgent])
    } else if allowed && screen == Screen::Reports {
        let ask = crate::explorer_ui::refresh(&state);
        (State { screen, ..state }, vec![Effect::BuildReports, ask])
    } else if allowed && screen == Screen::Achievements {
        (State { screen, ..state }, vec![Effect::CheckAchievements])
    } else if allowed && screen == Screen::Board {
        (State { screen, ..state }, vec![Effect::LoadBoard])
    } else if allowed && screen == Screen::Rehearse {
        crate::setup::refresh_preview(State { screen, ..state })
    } else if allowed {
        (State { screen, ..state }, Vec::new())
    } else {
        (state, Vec::new())
    }
}
