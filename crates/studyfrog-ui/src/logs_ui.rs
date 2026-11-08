//! "Open logs folder" in Settings > Data (STORY-0122): ask the worker to show the folder in the
//! system's file manager, and say so, or why not. Pure.

use crate::effect::Effect;
use crate::notify::NotifyEffect;
use crate::state::{Notice, NoticeKind, State};

/// Messages about the logs folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogsMsg {
    /// Show the logs folder.
    Open,
    /// The file manager was started for this folder, or why not.
    Opened(Result<String, String>),
}

/// Handle a message; the state is changed in place.
pub fn update(mut state: State, msg: LogsMsg) -> (State, Vec<Effect>) {
    match msg {
        LogsMsg::Open => (state, vec![Effect::Notifications(NotifyEffect::OpenLogs)]),
        LogsMsg::Opened(result) => {
            let (kind, text) = match result {
                Ok(dir) => (NoticeKind::Info, format!("Opened the logs folder {dir}.")),
                Err(e) => (
                    NoticeKind::Error,
                    format!("Could not open the logs folder: {e}"),
                ),
            };
            state.notice = Some(Notice { kind, text });
            (state, Vec::new())
        }
    }
}
