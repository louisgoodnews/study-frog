//! The Console's work at the executor: writing the transcript to the vault's log file (STORY-0152), and the scheduled
//! jobs of `scripts.toml` (STORY-0151). Each answers with the console's or the job editor's message.

use std::fs;

use crate::console_ui::jobs::{JobsEffect, JobsMsg};
use crate::console_ui::{ConsoleEffect, ConsoleMsg};
use crate::exec::Worker;
use crate::msg::Msg;
use studyfrog_service::scripting::jobs::{history, read_config, rows_of, save, scripts_available};

/// The file the transcript is written to, inside the vault.
pub const LOG_FILE: &str = "logs/script_runs.log";

/// Perform a Console effect and answer with its message.
pub fn run(w: &mut Worker, effect: ConsoleEffect, now_ns: i64) -> Msg {
    match effect {
        ConsoleEffect::Export(text) => Msg::Console(ConsoleMsg::Exported(export(w, &text))),
        ConsoleEffect::Jobs(e) => jobs(e, now_ns),
    }
}

/// Write `text` to the vault's log file, replacing the last export.
fn export(w: &Worker, text: &str) -> Result<String, String> {
    let Some(session) = &w.session else {
        return Err("no vault is open".to_owned());
    };
    let path = session.root.join(LOG_FILE);
    path.parent()
        .map_or(Ok(()), fs::create_dir_all)
        .and_then(|_| fs::write(&path, text))
        .map(|_| path.display().to_string())
        .map_err(|e| e.to_string())
}

/// The scheduled jobs: the app's `scripts` folder and log, as the window's runner reads them.
fn jobs(e: JobsEffect, now_ns: i64) -> Msg {
    let Some(paths) = crate::runtime::paths() else {
        return Msg::Console(ConsoleMsg::Jobs(JobsMsg::Loaded(Err(
            "the app's folder is not known".into(),
        ))));
    };
    let events = crate::script_events::EVENT_KINDS;
    let now_ms = now_ns / 1_000_000;
    match e {
        JobsEffect::Load => {
            let cfg = read_config(&paths.scripts, events);
            let loaded = crate::console_ui::jobs::Loaded {
                rows: rows_of(
                    &cfg,
                    studyfrog_core::time::UnixMs(now_ms),
                    studyfrog_service::timezone::local_offset_minutes(
                        studyfrog_core::time::UnixMs(now_ms),
                    ),
                ),
                errors: cfg.errors.clone(),
                python: cfg.python.clone(),
                scripts: scripts_available(&paths.scripts),
            };
            Msg::Console(ConsoleMsg::Jobs(JobsMsg::Loaded(Ok(loaded))))
        }
        JobsEffect::Save(rows, python) => {
            let saved = save(&paths.scripts, &rows, &python, events);
            Msg::Console(ConsoleMsg::Jobs(JobsMsg::Saved(saved)))
        }
        JobsEffect::History(name) => Msg::Console(ConsoleMsg::Jobs(JobsMsg::HistoryLoaded(
            history(&paths.logs, &name, 20),
        ))),
    }
}
