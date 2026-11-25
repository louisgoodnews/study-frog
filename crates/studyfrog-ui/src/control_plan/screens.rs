//! The plans of the board, diary and reports commands. Each opens its screen first (the screen loads
//! its data in the background), then makes the clicks: a card's column and title, a new entry's
//! fields and its save, the report tab.
use crate::msg::MsgVault;

use crate::control_cmd::Command;
use crate::control_cmd::board::BoardCmd;
use crate::control_cmd::diary::{DiaryCmd, DiaryNew};
use crate::control_cmd::wizard::{StepOp, WizardCmd};
use crate::control_plan::Plan;
use crate::diary_msg::DiaryMsg;
use crate::explorer_ui::ExplorerMsg;
use crate::kanban_msg::BoardMsg;
use crate::msg::Msg;
use crate::state::Screen;
use crate::view::wizard::types::WizardMsg;

fn open(screen: Screen) -> Vec<Msg> {
    vec![Msg::Vault(MsgVault::Navigate(screen))]
}

fn board(cmd: &BoardCmd) -> Result<Plan, String> {
    let open = open(Screen::Board);
    Ok(match cmd {
        BoardCmd::List | BoardCmd::Column(_) => vec![open],
        BoardCmd::Add { title, column } => {
            if title.trim().is_empty() {
                return Err("a card needs a title".to_owned());
            }
            let add = match column {
                Some(name) => BoardMsg::AddTo(name.clone()),
                None => BoardMsg::Add,
            };
            vec![
                open,
                vec![
                    Msg::Board(BoardMsg::NewTitle(title.clone())),
                    Msg::Board(add),
                ],
            ]
        }
    })
}

/// The new entry: the editor opens on its day, the fields are typed, then it is saved.
fn diary_add(n: &DiaryNew) -> Vec<Vec<Msg>> {
    let d = Msg::Diary;
    vec![
        open(Screen::Diary),
        vec![
            d(DiaryMsg::NewEntry(n.date)),
            d(DiaryMsg::UpdateDraftTitle(n.title.clone())),
            d(DiaryMsg::UpdateDraftContent(n.text.clone())),
            d(DiaryMsg::UpdateDraftTags(n.tags.clone())),
            d(DiaryMsg::UpdateDraftStacks(n.stacks.clone())),
            d(DiaryMsg::UpdateDraftMinutes(n.minutes)),
            d(DiaryMsg::SaveDraft),
        ],
    ]
}

fn diary(cmd: &DiaryCmd) -> Plan {
    match cmd {
        DiaryCmd::List { .. } => vec![open(Screen::Diary)],
        DiaryCmd::Open(uid) => vec![
            open(Screen::Diary),
            vec![Msg::Diary(DiaryMsg::EditEntry(*uid))],
        ],
        DiaryCmd::Add(n) => diary_add(n),
    }
}

fn reports(tab: Option<crate::explorer_ui::ReportTab>) -> Plan {
    let mut phases = vec![open(Screen::Reports)];
    if let Some(tab) = tab {
        phases.push(vec![Msg::Explorer(ExplorerMsg::Tab(tab))]);
    }
    phases
}

/// The plan for the board, diary and reports commands; any other command has an empty plan.
pub fn plan(cmd: &Command) -> Result<Plan, String> {
    Ok(match cmd {
        Command::Board(c) => board(c)?,
        Command::Diary(c) => diary(c),
        Command::Reports(tab) => reports(*tab),
        Command::Audit | Command::Ollama(_) => vec![vec![ollama_check()]],
        Command::Wizard(WizardCmd::Step(op)) => vec![vec![wizard_step(*op)]],
        _ => Vec::new(),
    })
}

/// The local-model setup's check of this computer and Ollama (STORY-0206); the answer reads it.
fn ollama_check() -> Msg {
    crate::agent_types::ollama::msg(crate::agent_types::ollama::OllamaMsg::Refresh)
}

/// A move of the setup wizard, as the message its own buttons send (STORY-0212). No step starts a download.
fn wizard_step(op: StepOp) -> Msg {
    studyfrog_log::log_info!("ui.control", "wizard step command received: {:?}", op);
    Msg::Wizard(match op {
        StepOp::Next => WizardMsg::Next,
        StepOp::Previous => WizardMsg::Previous,
        StepOp::Skip => WizardMsg::Skip,
        StepOp::Goto(step) => WizardMsg::Goto(step),
    })
}
