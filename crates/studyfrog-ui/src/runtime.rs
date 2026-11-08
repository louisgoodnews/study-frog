//! The worker thread: runs effects off the UI thread and sends the results back as messages.

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, OnceLock};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use eframe::egui;
use studyfrog_config::paths::{AppPaths, HOME_ENV, app_home, app_paths};
use studyfrog_store::app_config::app_config_dir;
use studyfrog_store::app_migrate::{migrate_legacy_config, secure_secrets};

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
    /// Effects sent and not yet finished (the control protocol waits for zero).
    pub pending: Arc<AtomicUsize>,
}

/// Hand an effect to the worker, counting it as pending until the worker is done with it.
pub fn send_effect(rt: &Runtime, effect: Effect) {
    rt.pending.fetch_add(1, Ordering::SeqCst);
    if rt.effects.send(effect).is_err() {
        rt.pending.fetch_sub(1, Ordering::SeqCst);
    }
}

fn now_ns() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_nanos()).unwrap_or(i64::MAX))
}

fn env(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

/// Where the application keeps its files, with the old config directory copied in on first use.
/// An explicit `STUDYFROG_HOME` is an isolated home, so nothing is copied into it.
fn resolve_paths() -> Option<AppPaths> {
    let (own, data, home) = (env(HOME_ENV), env("XDG_DATA_HOME"), env("HOME"));
    let paths = app_paths(&app_home(own.as_deref(), data.as_deref(), home.as_deref())?);
    let legacy = app_config_dir(env("XDG_CONFIG_HOME").as_deref(), home.as_deref());
    if let Some(legacy) = legacy.filter(|l| own.is_none() && *l != paths.config) {
        let _ = migrate_legacy_config(&legacy, &paths.config);
    }
    let _ = secure_secrets(&paths.config);
    Some(paths)
}

/// The application's directories, found and migrated once per run.
pub fn paths() -> Option<AppPaths> {
    static PATHS: OnceLock<Option<AppPaths>> = OnceLock::new();
    PATHS.get_or_init(resolve_paths).clone()
}

pub(crate) fn config_dir() -> Option<PathBuf> {
    paths().map(|p| p.config)
}

/// Start the worker. It repaints the window whenever it has produced messages.
pub fn spawn(ctx: egui::Context, host: Host) -> Runtime {
    let (effects, effect_rx) = channel::<Effect>();
    let (msg_tx, msgs) = channel::<Msg>();
    let pending = Arc::new(AtomicUsize::new(0));
    let counter = pending.clone();
    thread::spawn(move || {
        let paths = paths();
        let models = paths.as_ref().map(|p| p.models.clone());
        let mut worker = worker_with_host(config_dir(), models, host);
        worker.state_dir = paths.map(|p| p.logs);
        while let Ok(effect) = effect_rx.recv() {
            for msg in execute(&mut worker, effect, now_ns()) {
                if msg_tx.send(msg).is_err() {
                    return;
                }
            }
            counter.fetch_sub(1, Ordering::SeqCst);
            ctx.request_repaint();
        }
    });
    Runtime {
        effects,
        msgs,
        pending,
    }
}
