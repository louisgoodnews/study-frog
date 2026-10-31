//! The worker thread: runs effects off the UI thread and sends the results back as messages.

use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use eframe::egui;
use studyfrog_slm::models::models_dir;
use studyfrog_store::app_config::app_config_dir;

use crate::effect::Effect;
use crate::exec::{execute, worker_with_host};
use crate::host::Host;
use crate::msg::Msg;

/// Channels to and from the worker.
pub struct Runtime {
    /// Effects to perform.
    pub effects: Sender<Effect>,
    /// Results to apply.
    pub msgs: Receiver<Msg>,
}

fn now_ns() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_nanos()).unwrap_or(i64::MAX))
}

fn config_dir() -> Option<std::path::PathBuf> {
    let xdg = std::env::var("XDG_CONFIG_HOME").ok();
    let home = std::env::var("HOME").ok();
    app_config_dir(xdg.as_deref(), home.as_deref())
}

/// `$XDG_STATE_HOME/studyfrog`, else `$HOME/.local/state/studyfrog`: where the agent logs its calls.
fn state_dir() -> Option<std::path::PathBuf> {
    let non_empty = |v: Option<String>| v.filter(|s| !s.is_empty()).map(std::path::PathBuf::from);
    let base = non_empty(std::env::var("XDG_STATE_HOME").ok()).or_else(|| {
        non_empty(std::env::var("HOME").ok()).map(|h| h.join(".local").join("state"))
    })?;
    Some(base.join("studyfrog"))
}

/// Start the worker. It repaints the window whenever it has produced messages.
pub fn spawn(ctx: egui::Context, host: Host) -> Runtime {
    let (effects, effect_rx) = channel::<Effect>();
    let (msg_tx, msgs) = channel::<Msg>();
    thread::spawn(move || {
        let models = {
            let xdg = std::env::var("XDG_DATA_HOME").ok();
            let home = std::env::var("HOME").ok();
            models_dir(xdg.as_deref(), home.as_deref())
        };
        let mut worker = worker_with_host(config_dir(), models, host);
        worker.state_dir = state_dir();
        while let Ok(effect) = effect_rx.recv() {
            for msg in execute(&mut worker, effect, now_ns()) {
                if msg_tx.send(msg).is_err() {
                    return;
                }
            }
            ctx.request_repaint();
        }
    });
    Runtime { effects, msgs }
}
