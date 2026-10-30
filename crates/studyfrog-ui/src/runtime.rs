//! The worker thread: runs effects off the UI thread and sends the results back as messages.

use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use eframe::egui;
use studyfrog_store::app_config::app_config_dir;

use crate::effect::Effect;
use crate::exec::{execute, worker_new};
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

/// Start the worker. It repaints the window whenever it has produced messages.
pub fn spawn(ctx: egui::Context) -> Runtime {
    let (effects, effect_rx) = channel::<Effect>();
    let (msg_tx, msgs) = channel::<Msg>();
    thread::spawn(move || {
        let mut worker = worker_new(config_dir());
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
