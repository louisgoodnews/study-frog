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
}

/// Messages of the explorer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExplorerMsg {
    /// The other tab.
    Tab(ReportTab),
    /// Look at a stack, an item or the whole vault.
    Open(Scope),
    /// Look further back or not.
    Period(Period),
    /// The numbers arrived.
    Loaded(Result<Box<Explorer>, String>),
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
