#![allow(dead_code, unused_imports, clippy::unwrap_used, clippy::expect_used)] // shared test helpers
//! Helpers for the control protocol tests: a headless engine on a sample vault, and in-memory
//! connections to it.

pub use std::io::{Cursor, Write};
pub use std::path::{Path, PathBuf};
pub use std::sync::mpsc::channel;
pub use std::sync::{Arc, Mutex};

pub use serde_json::{Value, json};
pub use studyfrog_ui::control_cmd::{ControlPolicy, policy_default};
pub use studyfrog_ui::control_conn::{ConnCtx, Inbox, Job, serve_connection};
pub use studyfrog_ui::control_engine::{Engine, engine_loop, handle, run_phase};
pub use studyfrog_ui::control_logs::SharedOut;
pub use studyfrog_ui::control_parse::parse_request;

pub use crate::common::{TempDir, sample_vault, temp_dir};
use studyfrog_ui::exec::worker_new;
use studyfrog_ui::model::Model;
use studyfrog_ui::msg::Msg;

/// The French sample vault's card A.
pub const CARD_A: &str = "0000000000000000000000000A";

/// A writer that keeps what was written, readable from elsewhere.
pub struct Buf(pub Arc<Mutex<Vec<u8>>>);

impl Write for Buf {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(data);
        Ok(data.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// A clock that moves one second per reading.
pub fn ticking() -> studyfrog_ui::control_engine::Clock {
    let mut t = 100.0;
    Box::new(move || {
        t += 1.0;
        (t, 1_800_000_000_000 + (t * 1000.0) as i64)
    })
}

/// A headless engine with the sample vault open; the log goes to `logs`.
pub fn engine(vault: &Path, logs: &Path, policy: ControlPolicy) -> Engine {
    let mut worker = worker_new(None);
    worker.state_dir = Some(logs.to_path_buf());
    let mut model = Model::default();
    model.state.app.tutorial_offered = true;
    let mut e = Engine {
        model,
        worker,
        policy,
        logs: Some(logs.to_path_buf()),
        clock: ticking(),
    };
    run_phase(&mut e, vec![Msg::OpenRecent(vault.display().to_string())]);
    assert!(e.model.state.vault.is_some());
    e
}

/// Context for a connection that hands its jobs to `inbox`.
pub fn conn_ctx(inbox: Inbox, logs: &Path, wait: fn()) -> ConnCtx {
    ConnCtx {
        inbox,
        wake: Arc::new(|| {}),
        logs: Some(logs.to_path_buf()),
        wait,
    }
}

/// Parse every line of `text` as JSON.
pub fn lines(text: &str) -> Vec<Value> {
    text.lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

/// Run a whole session: `input` (one request per line) through a connection on another thread,
/// answered by `e` on this one; the answers in order.
pub fn session(e: &mut Engine, logs: &Path, input: &str) -> Vec<Value> {
    let (inbox, jobs) = channel();
    let ctx = conn_ctx(inbox, logs, || {});
    let written = Arc::new(Mutex::new(Vec::new()));
    let out: SharedOut = Arc::new(Mutex::new(Box::new(Buf(written.clone()))));
    let text = input.to_owned();
    let reader = std::thread::spawn(move || {
        serve_connection(&ctx, Cursor::new(text), &out);
        ctx.inbox.send(Job::Quit).unwrap();
    });
    engine_loop(e, &jobs);
    reader.join().unwrap();
    let text = String::from_utf8(written.lock().unwrap().clone()).unwrap();
    lines(&text)
}

/// The answer to one request line.
pub fn ask(e: &mut Engine, line: &str) -> Value {
    let req = parse_request(line).unwrap();
    serde_json::from_str(&handle(e, &req)).unwrap()
}
