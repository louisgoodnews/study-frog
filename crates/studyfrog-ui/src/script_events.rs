//! Lifecycle events (ADR 0013): what happened in an instance, derived from the messages it applies and
//! from a probe of the state before and after. Pure; publishing is in `control_events`.

use serde_json::{Value, json};
use studyfrog_core::run::{RunState, run_mode_name};
use studyfrog_core::uid::Uid;

use crate::control_state::ulid;
use crate::effect::Effect;
use crate::focus_ui::FocusMsg;
use crate::modal::ModalMsg;
use crate::model::{Model, apply};
use crate::msg::{EditAction, Msg};
use crate::recall_types::RecallMsg;
use crate::state::State;

/// Every kind of lifecycle event, in the order the help lists them.
pub const EVENT_KINDS: &[&str] = &[
    "vault_opened",
    "vault_closed",
    "item_created",
    "item_saved",
    "item_deleted",
    "rehearsal_started",
    "rehearsal_stepped",
    "rehearsal_finished",
    "exam_finished",
    "diary_entry_saved",
    "import_finished",
    "backup_created",
    "focus_block_finished",
    "script_finished",
];

/// One event: its kind and what it is about.
#[derive(Debug, Clone, PartialEq)]
pub struct Lifecycle {
    /// One of [`EVENT_KINDS`].
    pub kind: &'static str,
    /// Details (ids, paths, counts).
    pub data: Value,
}

/// What is compared before and after a change: the open vault and the run on screen.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Probe {
    /// The open vault's folder.
    pub vault: Option<String>,
    /// The run's id and lifecycle state.
    pub run: Option<(Uid, RunState)>,
}

/// The probe of `state`.
pub fn probe(state: &State) -> Probe {
    Probe {
        vault: state.vault.as_ref().map(|v| v.root.clone()),
        run: state
            .rehearse
            .run
            .as_ref()
            .map(|r| (r.session.run.uid, r.session.run.state)),
    }
}

fn event(kind: &'static str, data: Value) -> Lifecycle {
    Lifecycle { kind, data }
}

/// Events a message reports by itself, read before it is applied to `state`.
pub fn from_msg(msg: &Msg, state: &State) -> Vec<Lifecycle> {
    match msg {
        Msg::Modal(ModalMsg::Created(Ok(o))) => vec![event(
            "item_created",
            json!({"uid": o.created.uid.map(ulid), "title": o.created.title, "path": o.created.path}),
        )],
        Msg::Saved(Ok(_)) => state.editor.as_ref().map_or_else(Vec::new, |e| {
            vec![event(
                "item_saved",
                json!({"uid": ulid(e.uid), "title": e.form.title, "path": e.path}),
            )]
        }),
        Msg::Edit(EditAction::ConfirmDelete) => state.editor.as_ref().map_or_else(Vec::new, |e| {
            vec![event(
                "item_deleted",
                json!({"uid": ulid(e.uid), "title": e.form.title, "path": e.path}),
            )]
        }),
        Msg::RunReady(Ok(run)) => vec![event(
            "rehearsal_started",
            json!({
                "run": ulid(run.uid), "mode": run_mode_name(run.mode),
                "items": run.queue.len(),
            }),
        )],
        Msg::StepRecorded(Ok(())) => state.rehearse.run.as_ref().map_or_else(Vec::new, |r| {
            vec![event(
                "rehearsal_stepped",
                json!({
                    "run": ulid(r.session.run.uid), "events": r.events.len(),
                }),
            )]
        }),
        Msg::Exam(crate::exam_ui::ExamMsg::Graded(uid, Ok(res))) => vec![event(
            "exam_finished",
            json!({
                "exam": ulid(*uid), "items": res.items,
                "percent": res.percent, "passed": res.passed,
            }),
        )],
        Msg::Diary(crate::diary_msg::DiaryMsg::Saved(Ok(_))) => {
            vec![event("diary_entry_saved", json!({}))]
        }
        Msg::Imported(Ok(outcome)) => vec![event(
            "import_finished",
            json!({
                "created": outcome.report.created,
                "skipped": outcome.report.skipped.len(),
            }),
        )],
        Msg::Backup(crate::backup_ui::BackupMsg::Done(Ok(Some(path)))) => {
            vec![event("backup_created", json!({"path": path}))]
        }
        Msg::Recall(RecallMsg::Focus(FocusMsg::Recorded(Ok(r)))) => vec![event(
            "focus_block_finished",
            json!({
                "minutes": r.minutes, "today": r.today,
            }),
        )],
        _ => Vec::new(),
    }
}

fn vault_change(before: &Probe, after: &Probe) -> Vec<Lifecycle> {
    if before.vault == after.vault {
        return Vec::new();
    }
    let closed = before
        .vault
        .as_ref()
        .map(|root| event("vault_closed", json!({"root": root})));
    let opened = after
        .vault
        .as_ref()
        .map(|root| event("vault_opened", json!({"root": root})));
    closed.into_iter().chain(opened).collect()
}

fn run_finished(before: &Probe, state: &State) -> Option<Lifecycle> {
    let run = state.rehearse.run.as_ref()?;
    let (uid, now) = (run.session.run.uid, run.session.run.state);
    let was = before.run.filter(|(u, _)| *u == uid).map(|(_, s)| s);
    if now != RunState::Finished || was == Some(RunState::Finished) {
        return None;
    }
    Some(event(
        "rehearsal_finished",
        json!({
            "run": ulid(uid), "mode": run_mode_name(run.session.run.mode),
            "items": run.session.queue.len(), "events": run.events.len(),
        }),
    ))
}

/// Events a change of state shows: a vault opened or closed, a run that has just finished.
pub fn from_change(before: &Probe, after: &State) -> Vec<Lifecycle> {
    let mut out = vault_change(before, &probe(after));
    out.extend(run_finished(before, after));
    out
}

/// Apply `msg` to the model and add the events it caused to `out`; the effects it asks for.
pub fn apply_observed(model: &mut Model, msg: Msg, out: &mut Vec<Lifecycle>) -> Vec<Effect> {
    let before = probe(&model.state);
    out.extend(from_msg(&msg, &model.state));
    let effects = apply(model, msg);
    out.extend(from_change(&before, &model.state));
    effects
}

/// The event as the JSON a subscriber receives: kind, time and details.
pub fn event_json(ev: &Lifecycle, at_ms: i64) -> Value {
    json!({"kind": ev.kind, "at_ms": at_ms, "data": ev.data})
}
