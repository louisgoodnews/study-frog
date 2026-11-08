#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The window's side of the control protocol, frame by frame: a command's phases are applied one per
//! idle frame and it is answered once the worker has settled.

mod common;
mod control_common;

use control_common::*;
use eframe::egui;
use studyfrog_ui::control_window::{WindowControl, pump};

fn control(policy: ControlPolicy) -> (WindowControl, Inbox) {
    let (inbox, jobs) = channel();
    let c = WindowControl {
        jobs,
        pending: None,
        policy,
        logs: None,
        ctx: None,
        #[cfg(unix)]
        guard: None,
    };
    (c, inbox)
}

fn request(inbox: &Inbox, line: &str) -> std::sync::mpsc::Receiver<String> {
    let (tx, rx) = channel();
    inbox
        .send(Job::Run(parse_request(line).unwrap(), tx))
        .unwrap();
    rx
}

#[test]
fn a_command_is_answered_after_its_phases_ran_and_the_worker_settled() {
    let (_vault, mut m, mut w) = common::flow::opened("ctl-window");
    let (mut c, inbox) = control(policy_default());
    let ctx = egui::Context::default();
    let rx = request(&inbox, "item edit 0000000000000000000000000A title Neu");
    let fx = pump(&mut c, &mut m, &ctx, true);
    assert!(!fx.is_empty(), "the first phase loads the item");
    assert!(rx.try_recv().is_err(), "not answered while work is pending");
    assert!(
        pump(&mut c, &mut m, &ctx, false).is_empty(),
        "a busy frame waits"
    );
    common::run_to_idle(&mut m, &mut w, fx);
    let fx = pump(&mut c, &mut m, &ctx, true);
    assert!(!fx.is_empty(), "the second phase edits and saves");
    common::run_to_idle(&mut m, &mut w, fx);
    assert!(pump(&mut c, &mut m, &ctx, true).is_empty());
    let answer: Value = serde_json::from_str(&rx.try_recv().unwrap()).unwrap();
    assert_eq!(answer["result"]["editor"]["title"], "Neu");
    assert!(c.pending.is_none());
}

#[test]
fn refused_and_invalid_commands_are_answered_at_once() {
    let (_vault, mut m, _w) = common::flow::opened("ctl-window-refuse");
    let (mut c, inbox) = control(policy_default());
    let ctx = egui::Context::default();
    let rx = request(&inbox, "item delete 0000000000000000000000000A");
    pump(&mut c, &mut m, &ctx, true);
    assert!(rx.try_recv().unwrap().contains("destructive"));
    let rx = request(&inbox, "stack open german");
    pump(&mut c, &mut m, &ctx, true);
    assert!(rx.try_recv().unwrap().contains("no stack in folder german"));
    let rx = request(&inbox, "state");
    pump(&mut c, &mut m, &ctx, true);
    pump(&mut c, &mut m, &ctx, true);
    assert!(rx.try_recv().unwrap().contains("\"screen\":\"home\""));
}
