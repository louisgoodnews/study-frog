//! The performance explorer on the reports screen: what is shown and how it changes. Pure.

use studyfrog_service::explore::{Explorer, Period, Scope};

use crate::effect::Effect;
use crate::state::{Notice, NoticeKind, State};

/// The two halves of the reports screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReportTab {
    /// How the user did, by vault, stack and item.
    #[default]
    Performance,
    /// Retention, answer times and the forecast.
    Memory,
    /// How familiar the user is with the items.
    Familiarity,
    /// Self-assessment calibration against measured performance.
    Calibration,
    /// The history of runs.
    Runs,
    /// Recall by hour and weekday, and fatigue within a run.
    Habits,
}

/// The tabs by the word that names them on the command line (`--screen reports:<word>`).
pub const TAB_WORDS: [(&str, ReportTab); 6] = [
    ("performance", ReportTab::Performance),
    ("memory", ReportTab::Memory),
    ("familiarity", ReportTab::Familiarity),
    ("calibration", ReportTab::Calibration),
    ("runs", ReportTab::Runs),
    ("habits", ReportTab::Habits),
];

/// The tab a word names.
pub fn tab_named(word: &str) -> Option<ReportTab> {
    TAB_WORDS.iter().find(|(w, _)| *w == word).map(|(_, t)| *t)
}

/// What the explorer is looking at and what it found.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExplorerState {
    /// The tab on screen.
    pub tab: ReportTab,
    /// What is being looked at.
    pub scope: Scope,
    /// How far back.
    pub period: Period,
    /// The latest numbers for the scope and period.
    pub data: Option<Box<Explorer>>,
    /// Only runs of this mode in the history of runs.
    pub mode: Option<studyfrog_core::run::RunMode>,
    /// Reviews an hour or weekday needs before it can be a peak or trough (0: the report's default).
    pub min_bucket: u32,
}

/// Messages of the explorer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExplorerMsg {
    /// The other tab.
    Tab(ReportTab),
    /// Look at a stack, an item or the whole vault.
    Open(Scope),
    /// Show only runs of a mode (`None`: all).
    Mode(Option<studyfrog_core::run::RunMode>),
    /// Look further back or not.
    Period(Period),
    /// The numbers arrived.
    Loaded(Result<Box<Explorer>, String>),
    /// Reviews an hour or weekday needs before it is recommended.
    MinBucket(u32),
}

fn ask(state: State, scope: Scope, period: Period) -> (State, Vec<Effect>) {
    let explorer = ExplorerState {
        scope: scope.clone(),
        period,
        ..state.explorer.clone()
    };
    (
        State { explorer, ..state },
        vec![Effect::Explore { scope, period }],
    )
}

/// The effect that refreshes the explorer with its current scope and period.
pub fn refresh(state: &State) -> Effect {
    Effect::Explore {
        scope: state.explorer.scope.clone(),
        period: state.explorer.period,
    }
}

/// Handle the explorer's messages.
pub fn update(state: State, msg: ExplorerMsg) -> (State, Vec<Effect>) {
    match msg {
        ExplorerMsg::Mode(mode) => {
            let explorer = ExplorerState {
                mode,
                ..state.explorer.clone()
            };
            (State { explorer, ..state }, Vec::new())
        }
        ExplorerMsg::MinBucket(n) => {
            let explorer = ExplorerState {
                min_bucket: n,
                ..state.explorer.clone()
            };
            (State { explorer, ..state }, Vec::new())
        }
        ExplorerMsg::Tab(tab) => {
            let explorer = ExplorerState {
                tab,
                ..state.explorer.clone()
            };
            (State { explorer, ..state }, Vec::new())
        }
        ExplorerMsg::Open(scope) => {
            let period = state.explorer.period;
            ask(state, scope, period)
        }
        ExplorerMsg::Period(period) => {
            let scope = state.explorer.scope.clone();
            ask(state, scope, period)
        }
        ExplorerMsg::Loaded(Ok(data)) => {
            let explorer = ExplorerState {
                data: Some(data),
                ..state.explorer.clone()
            };
            (State { explorer, ..state }, Vec::new())
        }
        ExplorerMsg::Loaded(Err(e)) => {
            let notice = Some(Notice {
                kind: NoticeKind::Error,
                text: format!("Could not compute the performance: {e}"),
            });
            (State { notice, ..state }, Vec::new())
        }
    }
}
