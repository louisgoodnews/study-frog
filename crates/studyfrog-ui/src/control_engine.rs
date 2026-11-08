//! The headless side of the control protocol: the model and a worker in one thread, driven by
//! requests. Each phase of a command is applied and its effects are executed until nothing is left,
//! so an answer always describes the settled state. No egui and no window.

use std::path::PathBuf;

use crate::control_cmd::{Command, ControlPolicy, Request, allowed};
use crate::control_plan::plan_for;
use crate::control_reply::{err_line, ok_line, result_for};
use crate::effect::Effect;
use crate::exec::{Worker, execute};
use crate::model::{Model, apply};
use crate::msg::Msg;
use crate::notify::log_new;

/// Rounds of effects executed for one phase before giving up waiting (a safety net; work normally
/// settles in a few rounds).
pub const MAX_ROUNDS: usize = 200;

/// The time: seconds on a monotonic clock, and the wall clock in Unix milliseconds.
pub type Clock = Box<dyn FnMut() -> (f64, i64) + Send>;

/// A headless instance.
pub struct Engine {
    /// The UI state.
    pub model: Model,
    /// Executes the effects.
    pub worker: Worker,
    /// What requests may change.
    pub policy: ControlPolicy,
    /// Where command failures are logged (`logs/` of the application home).
    pub logs: Option<PathBuf>,
    /// The time, read before each phase.
    pub clock: Clock,
}

fn set_time(e: &mut Engine) {
    let (now, wall) = (e.clock)();
    e.model.state.now = now;
    e.model.state.wall_ms = wall;
}

/// Apply `msg` and note a notice it raised for the log.
fn apply_logged(model: &mut Model, msg: Msg) -> Vec<Effect> {
    let before = model.state.notice.clone();
    let mut effects = apply(model, msg);
    effects.extend(log_new(&before, &model.state));
    effects
}

/// Execute effects and apply their messages until there is nothing left to do.
pub fn settle(e: &mut Engine, mut effects: Vec<Effect>) {
    for _ in 0..MAX_ROUNDS {
        if effects.is_empty() {
            return;
        }
        let now_ns = e.model.state.wall_ms.saturating_mul(1_000_000);
        let msgs: Vec<Msg> = effects
            .drain(..)
            .flat_map(|fx| execute(&mut e.worker, fx, now_ns))
            .collect();
        effects = msgs
            .into_iter()
            .flat_map(|m| apply_logged(&mut e.model, m))
            .collect();
    }
}

/// Apply one phase of messages and wait for its work.
pub fn run_phase(e: &mut Engine, msgs: Vec<Msg>) {
    set_time(e);
    let effects: Vec<Effect> = msgs
        .into_iter()
        .flat_map(|m| apply_logged(&mut e.model, m))
        .collect();
    settle(e, effects);
}

/// Log why a command failed, as an error trace of the control protocol.
pub fn log_failure(logs: Option<&PathBuf>, wall_ms: i64, text: &str) {
    use studyfrog_service::logging::{LogEntry, LogLevel, MAX_LOG_BYTES, log_append};
    let Some(dir) = logs else { return };
    let entry = LogEntry {
        at_ms: wall_ms,
        level: LogLevel::Warn,
        source: "control".to_owned(),
        message: text.to_owned(),
    };
    let _ = log_append(dir, &entry, MAX_LOG_BYTES);
}

fn refuse(e: &mut Engine, req: &Request, why: &str) -> String {
    set_time(e);
    log_failure(e.logs.as_ref(), e.model.state.wall_ms, why);
    err_line(&req.id, why)
}

/// Answer one request (not `logs`, which the connection serves itself).
pub fn handle(e: &mut Engine, req: &Request) -> String {
    if let Err(why) = allowed(&e.policy, &req.cmd) {
        return refuse(e, req, &why);
    }
    if let Command::Screenshot(_) = req.cmd {
        let why = "a headless instance has no window to take a picture of; attach to a running window with `studyfrog ctl`";
        return refuse(e, req, why);
    }
    let before = e.model.state.notice.clone();
    let plan = match plan_for(&e.model.state, &req.cmd) {
        Ok(plan) => plan,
        Err(why) => return refuse(e, req, &why),
    };
    for phase in plan {
        run_phase(e, phase);
    }
    match result_for(&e.model.state, &req.cmd, &before) {
        Ok(v) => ok_line(&req.id, v),
        Err(why) => refuse(e, req, &why),
    }
}

/// Answer jobs until a `Quit` arrives or every sender is gone.
pub fn engine_loop(e: &mut Engine, jobs: &std::sync::mpsc::Receiver<crate::control_conn::Job>) {
    while let Ok(crate::control_conn::Job::Run(req, reply)) = jobs.recv() {
        let _ = reply.send(handle(e, &req));
    }
}
