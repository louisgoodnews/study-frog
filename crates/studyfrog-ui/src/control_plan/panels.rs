//! The plans of the screens that had only `nav` (achievements, graph, assistant, settings, match,
//! folders, the PDF viewer's open) and of `stack delete`. Each opens its screen, then makes the
//! change; `stack delete` asks and confirms the way the folders screen does.

use studyfrog_md::ulid::format_ulid;

use crate::achievements::{AchievementsMsg, Filter};
use crate::agent_types::AgentMsg;
use crate::app_settings_ui::SettingsTabMsg;
use crate::control_cmd::screens::{
    AchFilter, AchievementsCmd, AssistantCmd, FoldersCmd, MatchCmd, ScreenCmd, SettingsCmd,
};
use crate::control_plan::{Plan, resolve_stack};
use crate::folders_types::FoldersMsg;
use crate::match_ui::MatchMsg;
use crate::msg::Msg;
use crate::pdf_types::PdfMsg;
use crate::state::{Screen, State};

fn open(screen: Screen) -> Vec<Msg> {
    vec![Msg::Navigate(screen)]
}

/// The achievements filter; a stack's filter is by its key (its uid as text).
fn filter_of(state: &State, f: &AchFilter) -> Result<Filter, String> {
    Ok(match f {
        AchFilter::All => Filter::All,
        AchFilter::Global => Filter::Global,
        AchFilter::Stack(folder) => Filter::Stack(format_ulid(resolve_stack(state, folder)?)),
    })
}

fn achievements(state: &State, cmd: &AchievementsCmd) -> Result<Plan, String> {
    let a = |m| Msg::Achievements(m);
    Ok(match cmd {
        AchievementsCmd::List => vec![open(Screen::Achievements), vec![a(AchievementsMsg::Wanted)]],
        AchievementsCmd::Filter(f) => vec![
            open(Screen::Achievements),
            vec![a(AchievementsMsg::Filter(filter_of(state, f)?))],
        ],
    })
}

fn assistant(state: &State, cmd: &AssistantCmd) -> Result<Plan, String> {
    Ok(match cmd {
        AssistantCmd::State => vec![open(Screen::Agent)],
        AssistantCmd::Send(text) => {
            if crate::agent_types::selected_provider(&state.agent).is_none() {
                return Err(
                    "not configured: no model is connected (add one under the assistant's Connections)"
                        .to_owned(),
                );
            }
            vec![
                open(Screen::Agent),
                vec![Msg::Agent(AgentMsg::ChatInput(text.clone()))],
                vec![Msg::Agent(AgentMsg::ChatSend)],
            ]
        }
    })
}

fn settings(cmd: &SettingsCmd) -> Plan {
    match cmd {
        SettingsCmd::Read => vec![open(Screen::Settings)],
        SettingsCmd::SlowAnswer(on) => vec![
            open(Screen::Settings),
            vec![Msg::SettingsTab(SettingsTabMsg::SlowAnswerNotice(*on))],
        ],
    }
}

fn matching(cmd: &MatchCmd) -> Plan {
    match cmd {
        MatchCmd::State => Vec::new(),
        MatchCmd::Start => vec![
            open(Screen::Library),
            vec![Msg::Match(MatchMsg::StartFromVault)],
        ],
    }
}

fn folders(state: &State, cmd: &FoldersCmd) -> Plan {
    match cmd {
        FoldersCmd::List => vec![open(Screen::Folders)],
        FoldersCmd::Open(path) => {
            let path = path.trim_matches('/').to_owned();
            let mut select = vec![Msg::Folders(FoldersMsg::Select(path.clone()))];
            if !state.folders.expanded.contains(&path) {
                select.push(Msg::Folders(FoldersMsg::ToggleExpand(path)));
            }
            vec![open(Screen::Folders), select]
        }
    }
}

/// The plan for a screen command.
pub fn plan(state: &State, cmd: &ScreenCmd) -> Result<Plan, String> {
    Ok(match cmd {
        ScreenCmd::Achievements(c) => achievements(state, c)?,
        ScreenCmd::Graph => vec![open(Screen::Graph)],
        ScreenCmd::Assistant(c) => assistant(state, c)?,
        ScreenCmd::Settings(c) => settings(c),
        ScreenCmd::Match(c) => matching(c),
        ScreenCmd::Folders(c) => folders(state, c),
        ScreenCmd::PdfOpen(path) => vec![vec![Msg::Pdf(PdfMsg::Open(path.clone(), 0))]],
    })
}

/// `stack delete FOLDER`: the folders screen asks (plans the delete), then the confirmation runs.
pub fn stack_delete(state: &State, folder: &str) -> Result<Plan, String> {
    resolve_stack(state, folder)?;
    let folder = folder.trim_matches('/').to_owned();
    Ok(vec![
        vec![Msg::Folders(FoldersMsg::AskDelete(folder))],
        vec![Msg::Folders(FoldersMsg::ConfirmDelete)],
    ])
}
