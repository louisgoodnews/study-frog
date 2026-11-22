#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The job editor's state (STORY-0151): opening reads the jobs, edits make the form dirty, a save asks for a checked
//! write, and a successful save reads the jobs again.

use studyfrog_service::scripting::config::Access;
use studyfrog_service::scripting::jobs::JobRow;
use studyfrog_ui::console_ui::jobs::{JobsEffect, JobsMsg, Loaded, update};
use studyfrog_ui::console_ui::{ConsoleEffect, ConsoleMsg};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::modal::Modal;
use studyfrog_ui::state::State;

fn job(name: &str) -> JobRow {
    JobRow {
        name: name.into(),
        script: format!("{name}.py"),
        args: Vec::new(),
        schedule: "0 9 * * *".into(),
        on: Vec::new(),
        timeout_secs: 60,
        access: Access::Read,
        enabled: true,
        next_run: Some("2027-03-01 09:00".into()),
    }
}

fn jobs_effect(e: JobsEffect) -> Vec<Effect> {
    vec![Effect::Console(ConsoleEffect::Jobs(e))]
}

fn editor(state: &State) -> &studyfrog_ui::console_ui::jobs::JobsEditor {
    match state.modal.as_ref().unwrap() {
        Modal::ScriptJobs(e) => e,
        other => panic!("not the job editor: {other:?}"),
    }
}

#[test]
fn opening_the_editor_reads_the_jobs_and_loaded_jobs_show() {
    let (state, fx) = update(State::default(), JobsMsg::Open);
    assert_eq!(fx, jobs_effect(JobsEffect::Load));
    let (state, _) = update(
        state,
        JobsMsg::Loaded(Ok(Loaded {
            rows: vec![job("health")],
            errors: vec!["job 2: unknown event x".into()],
            python: "python3".into(),
            scripts: vec!["health.py".into()],
        })),
    );
    let e = editor(&state);
    assert_eq!(e.rows.len(), 1);
    assert_eq!(e.errors.len(), 1);
    assert_eq!(e.adhoc_script, "health.py");
    assert!(!e.dirty);
}

#[test]
fn edits_make_the_form_dirty_and_a_save_asks_for_the_checked_write() {
    let (state, _) = update(State::default(), JobsMsg::Open);
    let (state, _) = update(
        state,
        JobsMsg::Loaded(Ok(Loaded {
            rows: vec![job("health")],
            errors: vec![],
            python: "python3".into(),
            scripts: vec![],
        })),
    );
    let (state, _) = update(state, JobsMsg::Enabled(0, false));
    let (state, _) = update(state, JobsMsg::Schedule(0, "30 8 * * mon-fri".into()));
    let (state, _) = update(state, JobsMsg::EventOn(0, "vault_opened".into(), true));
    let e = editor(&state);
    assert!(e.dirty);
    assert!(!e.rows[0].enabled);
    assert_eq!(e.rows[0].schedule, "30 8 * * mon-fri");
    assert_eq!(e.rows[0].on, vec!["vault_opened".to_owned()]);

    let (state, fx) = update(state, JobsMsg::Save);
    assert!(
        matches!(&fx[..], [Effect::Console(ConsoleEffect::Jobs(JobsEffect::Save(rows, python)))] if rows.len() == 1 && python == "python3")
    );
    // A save that was written reads the jobs again and is no longer dirty.
    let (state, fx) = update(state, JobsMsg::Saved(Ok(())));
    assert!(!editor(&state).dirty);
    assert_eq!(fx, jobs_effect(JobsEffect::Load));
    // A refused save keeps the changes and says why.
    let (state, _) = update(state, JobsMsg::Schedule(0, "bad".into()));
    let (state, _) = update(state, JobsMsg::Saved(Err("schedule: bad".into())));
    assert!(editor(&state).dirty);
    assert!(
        editor(&state)
            .notice
            .as_deref()
            .unwrap()
            .contains("Not saved")
    );
}

#[test]
fn selecting_a_job_asks_for_its_history() {
    let (state, _) = update(State::default(), JobsMsg::Open);
    let (state, _) = update(
        state,
        JobsMsg::Loaded(Ok(Loaded {
            rows: vec![job("health")],
            errors: vec![],
            python: "python3".into(),
            scripts: vec![],
        })),
    );
    let (state, fx) = update(state, JobsMsg::Select(0));
    assert_eq!(fx, jobs_effect(JobsEffect::History("health".into())));
    let (state, _) = update(
        state,
        JobsMsg::HistoryLoaded(vec!["script health finished".into()]),
    );
    assert_eq!(
        editor(&state).history,
        vec!["script health finished".to_owned()]
    );
    let _ = ConsoleMsg::OpenJobs;
}

#[test]
fn a_new_job_needs_a_name_and_a_script_and_is_added_switched_on() {
    let (state, _) = update(State::default(), JobsMsg::Open);
    let (state, _) = update(
        state,
        JobsMsg::Loaded(Ok(Loaded {
            rows: vec![job("health")],
            errors: vec![],
            python: "python3".into(),
            scripts: vec!["x.py".into()],
        })),
    );
    // No name yet: nothing is added.
    let (state, _) = update(state, JobsMsg::NewScript("x.py".into()));
    let (state, _) = update(state, JobsMsg::AddJob);
    assert_eq!(editor(&state).rows.len(), 1);
    // A name already used is refused.
    let (state, _) = update(state, JobsMsg::NewName("health".into()));
    let (state, _) = update(state, JobsMsg::AddJob);
    assert_eq!(editor(&state).rows.len(), 1);
    assert!(
        editor(&state)
            .notice
            .as_deref()
            .unwrap()
            .contains("already exists")
    );
    // A new name is added, switched on, selected and marked for saving.
    let (state, _) = update(state, JobsMsg::NewName("tidy".into()));
    let (state, _) = update(state, JobsMsg::AddJob);
    let e = editor(&state);
    assert_eq!(e.rows.len(), 2);
    assert_eq!(
        (
            e.rows[1].name.as_str(),
            e.rows[1].script.as_str(),
            e.rows[1].enabled
        ),
        ("tidy", "x.py", true)
    );
    assert_eq!(e.selected, Some(1));
    assert!(e.dirty && e.new_name.is_empty());
}
