//! One control connection (standard input and output, or a socket client): read request lines,
//! serve `logs` here, hand every other request to the owner of the model and write its answer.

use std::io::BufRead;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Sender, channel};

use studyfrog_config::paths::LOG_JSON_FILE;

use crate::control_cmd::{Command, ControlPolicy, LogsQuery, Request};
use crate::control_logs::{SharedOut, follow, read_entries, write_line};
use crate::control_reply::{err_line, ok_line};

/// What a connection asks of the owner of the model.
pub enum Job {
    /// Answer the request on this channel.
    Run(Request, Sender<String>),
    /// Answer it under this policy instead of the instance's (a script's own socket).
    RunScoped(Request, Sender<String>, ControlPolicy),
    /// The controlling input ended: stop (headless only; a window ignores it).
    Quit,
}

/// Sends jobs to the owner of the model.
pub type Inbox = Sender<Job>;
/// Wakes the owner when a job was sent (a window repaints).
pub type Wake = Arc<dyn Fn() + Send + Sync>;

/// What a connection needs besides its streams.
#[derive(Clone)]
pub struct ConnCtx {
    /// Where jobs go.
    pub inbox: Inbox,
    /// Called after each job was sent.
    pub wake: Wake,
    /// The application's `logs/` directory.
    pub logs: Option<PathBuf>,
    /// The pause between looks at a followed log.
    pub wait: fn(),
    /// The instance's lifecycle events, for `subscribe`.
    pub events: Option<crate::control_events::EventBus>,
    /// The policy of a script's own socket, used in place of the instance's.
    pub scope: Option<ControlPolicy>,
}

fn logs(
    ctx: &ConnCtx,
    id: &serde_json::Value,
    q: &LogsQuery,
    out: &SharedOut,
    alive: &Arc<AtomicBool>,
) -> String {
    let Some(dir) = &ctx.logs else {
        return err_line(id, "no application home is known, so there is no log");
    };
    let path = dir.join(LOG_JSON_FILE);
    let seen = std::fs::metadata(&path).map_or(0, |m| m.len());
    let entries = match read_entries(&path, q) {
        Ok(e) => e,
        Err(e) => return err_line(id, &format!("{}: {e}", path.display())),
    };
    if q.follow {
        let (q, out, alive, wait) = (q.clone(), out.clone(), alive.clone(), ctx.wait);
        std::thread::spawn(move || follow(&path, seen, &q, &out, &alive, wait));
    }
    let result = serde_json::json!({"entries": entries, "following": q.follow});
    ok_line(id, result)
}

fn ask(ctx: &ConnCtx, req: Request) -> String {
    let id = req.id.clone();
    let (tx, rx) = channel();
    let job = match ctx.scope {
        Some(policy) => Job::RunScoped(req, tx, policy),
        None => Job::Run(req, tx),
    };
    if ctx.inbox.send(job).is_err() {
        return err_line(&id, "the instance is shutting down");
    }
    (ctx.wake)();
    rx.recv()
        .unwrap_or_else(|_| err_line(&id, "the instance stopped before answering"))
}

/// The answer to one line.
pub fn answer(ctx: &ConnCtx, line: &str, out: &SharedOut, alive: &Arc<AtomicBool>) -> String {
    match crate::control_parse::parse_request(line) {
        Err((id, why)) => {
            let wall = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(0));
            crate::control_engine::log_failure(ctx.logs.as_ref(), wall, &why);
            err_line(&id, &why)
        }
        Ok(Request {
            id,
            cmd: Command::Logs(q),
        }) => logs(ctx, &id, &q, out, alive),
        Ok(Request {
            id,
            cmd: Command::Subscribe(kinds),
        }) => crate::control_events::serve_subscribe(ctx.events.as_ref(), &id, &kinds, out, alive),
        Ok(req) => ask(ctx, req),
    }
}

/// Serve requests from `input` until it ends or the output fails. Followed logs stop with it.
pub fn serve_connection<R: BufRead>(ctx: &ConnCtx, input: R, out: &SharedOut) {
    let alive = Arc::new(AtomicBool::new(true));
    for line in input.lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        if write_line(out, &answer(ctx, &line, out, &alive)).is_err() {
            break;
        }
    }
    alive.store(false, Ordering::Relaxed);
}
