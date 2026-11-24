//! Changing screens: which ones may be left, and what each one needs loaded.
use crate::effect::EffectAssistant;
use crate::effect::EffectBrowse;
use crate::effect::EffectEditor;
use crate::effect::EffectPlanner;
use crate::effect::EffectReports;
use crate::effect::EffectVault;
use crate::msg::MsgVault;

use crate::effect::Effect;
use crate::state::{Screen, State};

fn screen_effects(state: State, screen: Screen) -> (State, Vec<Effect>) {
    match screen {
        Screen::Graph => (
            State { screen, ..state },
            vec![Effect::Editor(EffectEditor::LoadGraph)],
        ),
        Screen::Calendar => (
            State { screen, ..state },
            vec![
                Effect::Planner(EffectPlanner::LoadSessions),
                Effect::LoadHomework,
            ],
        ),
        Screen::Homework => (State { screen, ..state }, vec![Effect::LoadHomework]),
        Screen::Folders => (
            State { screen, ..state },
            vec![
                Effect::Folders(crate::folders_types::FolderEffect::Scan),
                Effect::Browse(EffectBrowse::LoadLibrary),
                crate::pdf_library_ui::load(),
            ],
        ),
        Screen::Home | Screen::Library => (
            State { screen, ..state },
            vec![
                Effect::Vault(EffectVault::LoadHome),
                Effect::Planner(EffectPlanner::LoadSessions),
                Effect::Folders(crate::folders_types::FolderEffect::Scan),
                crate::pdf_library_ui::load(),
            ],
        ),
        Screen::Help => {
            let page = crate::help_ui::current(&state.help);
            crate::help_ui::open_at(state, page)
        }
        Screen::Agent => {
            let pages = Effect::Help(crate::help_types::HelpEffect::Load);
            let mcp = Effect::Agent(crate::agent_mcp_types::AgentEffect::LoadMcp);
            let tools = Effect::Agent(crate::agent_mcp_types::AgentEffect::LoadServers);
            (
                State { screen, ..state },
                vec![
                    Effect::Assistant(EffectAssistant::LoadAgent),
                    mcp,
                    tools,
                    pages,
                ],
            )
        }
        Screen::Reports => {
            let ask = crate::explorer_ui::refresh(&state);
            (
                State { screen, ..state },
                vec![Effect::Reports(EffectReports::BuildReports), ask],
            )
        }
        Screen::Settings => (
            State { screen, ..state },
            vec![Effect::Reports(EffectReports::BuildReports)],
        ),
        Screen::Prompts => (
            State { screen, ..state },
            vec![Effect::Assistant(EffectAssistant::LoadAgent)],
        ),
        Screen::Diary => (
            State { screen, ..state },
            vec![Effect::Diary(crate::diary_types::DiaryEffect::LoadEntries)],
        ),
        Screen::Achievements => (
            State { screen, ..state },
            vec![Effect::Reports(EffectReports::CheckAchievements)],
        ),
        Screen::Board => (
            State { screen, ..state },
            vec![Effect::Planner(EffectPlanner::LoadBoard)],
        ),
        Screen::Rehearse => crate::setup::refresh_preview(State { screen, ..state }),
        _ => (State { screen, ..state }, Vec::new()),
    }
}

/// Go to `screen` if allowed, asking for whatever data it shows.
pub fn navigate(state: State, screen: Screen) -> (State, Vec<Effect>) {
    let state = if screen == Screen::Editor {
        state
    } else {
        let then = crate::msg::Msg::Vault(MsgVault::Navigate(screen));
        let (state, waiting) = crate::editor_save::flush_before(state, then);
        if let Some(effects) = waiting {
            return (state, effects);
        }
        state
    };
    let allowed = (screen == Screen::Welcome || screen == Screen::Help || state.vault.is_some())
        && (screen == Screen::Editor || !crate::editor::has_unsaved_changes(&state));
    if allowed {
        // Entering the rehearse screen from another one is a new setup, which starts from Spaced (BUG-0054).
        let state = if screen == Screen::Rehearse && state.screen != Screen::Rehearse {
            crate::setup::start_setup(state)
        } else {
            state
        };
        screen_effects(state, screen)
    } else {
        (state, Vec::new())
    }
}
