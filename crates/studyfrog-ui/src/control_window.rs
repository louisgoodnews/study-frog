//! The control protocol inside the running window: requests from `run/control.sock` are taken one
//! at a time, each phase of a command is applied in a frame, and the answer is sent once the worker
//! has nothing left to do. `screenshot` asks the window for a picture.

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};

use eframe::egui;

use crate::control_cmd::{Command, ControlPolicy, Request, allowed};
use crate::control_conn::Job;
use crate::control_engine::log_failure;
use crate::control_plan::plan_for;
use crate::control_reply::{err_line, ok_line, result_for};
use crate::effect::Effect;
use crate::model::{Model, apply};
use crate::msg::Msg;
use crate::state::Notice;

/// Frames a command may wait for the worker before it is answered with what is there.
pub const MAX_FRAMES: u32 = 600;

/// A command being carried out.
pub struct Pending {
    /// The request.
    pub req: Request,
    /// Where the answer goes.
    pub reply: Sender<String>,
    /// Phases not applied yet.
    pub phases: VecDeque<Vec<Msg>>,
    /// The notice from before the command.
    pub before: Option<Notice>,
    /// Frames waited so far.
    pub frames: u32,
}

/// The window's side of the protocol.
pub struct WindowControl {
    /// Requests from the socket's connections.
    pub jobs: Receiver<Job>,
    /// The command being carried out.
    pub pending: Option<Pending>,
    /// What requests may change.
    pub policy: ControlPolicy,
    /// Where failures are logged.
    pub logs: Option<PathBuf>,
    /// Removes the socket when the window closes.
    #[cfg(unix)]
    pub guard: Option<crate::control_socket::ControlGuard>,
}

fn apply_phase(model: &mut Model, msgs: Vec<Msg>) -> Vec<Effect> {
    msgs.into_iter().flat_map(|m| apply(model, m)).collect()
}

fn fail(c: &WindowControl, model: &Model, req: &Request, reply: &Sender<String>, why: &str) {
    log_failure(c.logs.as_ref(), model.state.wall_ms, why);
    let _ = reply.send(err_line(&req.id, why));
}

/// Ask the window for a picture and answer once it is written.
fn screenshot(ctx: &egui::Context, req: Request, reply: Sender<String>, path: PathBuf) {
    let callback = egui::ScreenshotCallback::new(move |image| {
        let line = match crate::app_shot::save(&path, &image) {
            Ok(()) => ok_line(&req.id, serde_json::json!({"written": path})),
            Err(e) => err_line(&req.id, &format!("{}: {e}", path.display())),
        };
        let _ = reply.send(line);
    });
    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(callback));
}

fn start(c: &mut WindowControl, model: &mut Model, ctx: &egui::Context) -> Vec<Effect> {
    let Ok(Job::Run(req, reply)) = c.jobs.try_recv() else {
        return Vec::new();
    };
    if let Err(why) = allowed(&c.policy, &req.cmd) {
        fail(c, model, &req, &reply, &why);
        return Vec::new();
    }
    if let Command::Screenshot(path) = &req.cmd {
        let path = path.clone();
        screenshot(ctx, req, reply, path);
        return Vec::new();
    }
    let mut phases: VecDeque<Vec<Msg>> = match plan_for(&model.state, &req.cmd) {
        Ok(plan) => plan.into(),
        Err(why) => {
            fail(c, model, &req, &reply, &why);
            return Vec::new();
        }
    };
    let before = model.state.notice.clone();
    let effects = phases
        .pop_front()
        .map_or_else(Vec::new, |p| apply_phase(model, p));
    c.pending = Some(Pending {
        req,
        reply,
        phases,
        before,
        frames: 0,
    });
    effects
}

fn finish(c: &WindowControl, model: &Model, p: Pending) {
    match result_for(&model.state, &p.req.cmd, &p.before) {
        Ok(v) => {
            let _ = p.reply.send(ok_line(&p.req.id, v));
        }
        Err(why) => fail(c, model, &p.req, &p.reply, &why),
    }
}

/// One frame of the protocol. `idle` says the worker had nothing to do and sent nothing since the
/// last frame. Returns the effects of the phase applied in this frame.
pub fn pump(
    c: &mut WindowControl,
    model: &mut Model,
    ctx: &egui::Context,
    idle: bool,
) -> Vec<Effect> {
    let Some(mut p) = c.pending.take() else {
        return start(c, model, ctx);
    };
    p.frames += 1;
    ctx.request_repaint();
    if !idle && p.frames < MAX_FRAMES {
        c.pending = Some(p);
        return Vec::new();
    }
    match p.phases.pop_front() {
        Some(phase) => {
            p.frames = 0;
            let effects = apply_phase(model, phase);
            c.pending = Some(p);
            effects
        }
        None => {
            finish(c, model, p);
            Vec::new()
        }
    }
}

/// Serve `run/control.sock` for this window; `None` (with the reason on standard error) when there
/// is no application home or another instance serves it.
#[cfg(unix)]
pub fn window_control_new(
    ctx: &egui::Context,
    policy: ControlPolicy,
    events: crate::control_events::EventBus,
) -> Option<WindowControl> {
    use crate::control_conn::ConnCtx;
    use crate::control_socket::{bind, publish, serve};
    let p = crate::runtime::paths()?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(0));
    let bound = bind(&p.run).and_then(|l| Ok((l, publish(&p.run, "window", None, now)?)));
    let (listener, guard) = match bound {
        Ok(b) => b,
        Err(e) => {
            eprintln!("studyfrog: the control socket is not served: {e}");
            return None;
        }
    };
    let (inbox, jobs) = std::sync::mpsc::channel();
    let repaint = ctx.clone();
    let wake: crate::control_conn::Wake = std::sync::Arc::new(move || repaint.request_repaint());
    let logs = Some(p.logs.clone());
    serve(
        listener,
        ConnCtx {
            inbox,
            wake,
            logs: logs.clone(),
            wait: crate::cli::pause,
            events: Some(events),
        },
    );
    Some(WindowControl {
        jobs,
        pending: None,
        policy,
        logs,
        guard: Some(guard),
    })
}

/// No socket on this system.
#[cfg(not(unix))]
pub fn window_control_new(
    _: &egui::Context,
    _: ControlPolicy,
    _: crate::control_events::EventBus,
) -> Option<WindowControl> {
    None
}
