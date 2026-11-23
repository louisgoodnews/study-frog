//! `ui tree`, `ui find LABEL` and `ui click LABEL` in the window. The protocol asks egui for the
//! frame's widget tree and answers from it. A click is queued; it is answered after the frame that
//! took it has been drawn and the worker is idle.

use std::path::PathBuf;
use std::sync::mpsc::Sender;

use eframe::egui;
use serde_json::{Value, json};

use crate::control_cmd::ui::UiCmd;
use crate::control_cmd::{ControlPolicy, Request};
use crate::control_engine::log_failure;
use crate::control_reply::ui::{Widget, click_request, find_json, pick, tree_json};
use crate::control_reply::{err_line, ok_line};
use crate::control_window::MAX_FRAMES;
use crate::control_window::tree::{self, Shared};

/// A widget command waiting for its frame.
pub struct UiPending {
    /// The request.
    pub req: Request,
    /// Where the answer goes.
    pub reply: Sender<String>,
    /// What it asks for.
    pub cmd: UiCmd,
    /// What requests may change (a click on a destructive label needs the flag).
    pub policy: ControlPolicy,
    /// Answer only from a tree copied after this count.
    pub after: u64,
    /// Frames waited so far.
    pub frames: u32,
    /// The click's answer, once the click was queued.
    pub clicked: Option<Value>,
}

/// What the next step of a widget command is.
enum Next {
    /// Keep waiting (a click was queued).
    Wait,
    /// The answer.
    Done(Result<Value, String>),
}

/// Labels that delete or discard: a click on them needs `--control-allow-destructive`.
pub fn destructive(label: &str) -> bool {
    let l = label.to_lowercase();
    ["delete", "remove", "discard", "erase"]
        .iter()
        .any(|w| l.contains(w))
}

/// Start a widget command: the window draws a frame with its tree from now on.
pub fn start(
    tree: &Shared,
    ctx: &egui::Context,
    req: Request,
    reply: Sender<String>,
    cmd: UiCmd,
    policy: ControlPolicy,
) -> UiPending {
    let after = tree::want(tree, true);
    ctx.enable_accesskit();
    UiPending {
        req,
        reply,
        cmd,
        policy,
        after,
        frames: 0,
        clicked: None,
    }
}

fn widget_value(w: &Widget) -> Value {
    json!({"clicked": {"role": w.role, "label": w.label, "value": w.value, "enabled": w.enabled}})
}

/// The click: pick the widget, queue the click, then wait for the frame that took it.
fn click(
    tree: &Shared,
    p: &mut UiPending,
    frame: &egui::accesskit::TreeUpdate,
    label: &str,
    now: u64,
) -> Next {
    if let Some(done) = p.clicked.take() {
        return Next::Done(Ok(done));
    }
    let w = match pick(frame, label) {
        Ok(w) => w,
        Err(why) => return Next::Done(Err(why)),
    };
    if destructive(&w.label) && !p.policy.allow_destructive {
        let why = format!(
            "the widget {} deletes or discards: destructive commands are switched off (start with --control-allow-destructive)",
            w.label
        );
        return Next::Done(Err(why));
    }
    tree::queue_click(tree, click_request(w.id));
    p.clicked = Some(widget_value(&w));
    // The frame that takes the click is drawn after this one; its tree is the next copy after that.
    p.after = now + 1;
    Next::Wait
}

fn advance(tree: &Shared, p: &mut UiPending, now: u64) -> Next {
    let Some(frame) = tree::latest(tree) else {
        return Next::Done(Err("no widget tree was drawn yet".to_owned()));
    };
    match p.cmd.clone() {
        UiCmd::Tree => Next::Done(Ok(tree_json(&frame))),
        UiCmd::Find(label) => Next::Done(find_json(&frame, &label)),
        UiCmd::Click(label) => click(tree, p, &frame, &label, now),
    }
}

fn finish(
    tree: &Shared,
    ctx: &egui::Context,
    p: UiPending,
    result: Result<Value, String>,
    logs: Option<&PathBuf>,
    wall_ms: i64,
) {
    tree::want(tree, false);
    ctx.disable_accesskit();
    let line = match result {
        Ok(v) => ok_line(&p.req.id, v),
        Err(why) => {
            log_failure(logs, wall_ms, &why);
            err_line(&p.req.id, &why)
        }
    };
    let _ = p.reply.send(line);
}

/// One frame of a widget command. Returns the command again while it still waits; otherwise it is
/// answered here.
pub fn pump(
    tree: &Shared,
    ctx: &egui::Context,
    idle: bool,
    mut p: UiPending,
    logs: Option<&PathBuf>,
    wall_ms: i64,
) -> Option<UiPending> {
    p.frames += 1;
    ctx.request_repaint();
    let now = tree::captures(tree);
    let ready = (now > p.after && idle) || p.frames >= MAX_FRAMES;
    if !ready {
        return Some(p);
    }
    match advance(tree, &mut p, now) {
        Next::Wait => Some(p),
        Next::Done(result) => {
            finish(tree, ctx, p, result, logs, wall_ms);
            None
        }
    }
}
