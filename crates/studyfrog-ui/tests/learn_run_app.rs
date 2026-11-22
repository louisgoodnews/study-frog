#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Starting a custom learn-mode run from the app (STORY-0198): the editor saves a mode, and its start runs the
//! saved copy as a practice-only run that the run view takes over.

mod common;

use common::{NOW_NS, sample_vault};
use studyfrog_core::plan::ScheduleEffect;
use studyfrog_rehearsal::learn_graph::{Edge, LearnGraph, Node, NodeKind};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{execute, worker_with_host};
use studyfrog_ui::host::Host;
use studyfrog_ui::learn_editor::{LearnEffect, LearnMsg};
use studyfrog_ui::msg::Msg;

fn mode() -> LearnGraph {
    let node = |id: &str, kind| Node {
        id: id.into(),
        kind,
        label: id.into(),
    };
    LearnGraph {
        name: "Quick check".into(),
        nodes: vec![
            node("n1", NodeKind::Start),
            node("n2", NodeKind::Present),
            node("n3", NodeKind::Stop),
        ],
        edges: vec![
            Edge {
                from: "n1".into(),
                to: "n2".into(),
                when: None,
            },
            Edge {
                from: "n2".into(),
                to: "n3".into(),
                when: None,
            },
        ],
    }
}

#[test]
fn a_saved_mode_starts_as_a_practice_only_run() {
    let vault = sample_vault("learn-app");
    let mut w = worker_with_host(None, None, Host::default());
    execute(
        &mut w,
        Effect::OpenVault(vault.path.to_string_lossy().into_owned()),
        NOW_NS,
    );

    let saved = execute(
        &mut w,
        Effect::Learn(LearnEffect::Save(Box::new(mode()), Default::default())),
        NOW_NS,
    );
    let file = match &saved[..] {
        [Msg::Learn(LearnMsg::Saved(Ok(file)))] => file.clone(),
        other => panic!("the mode was not saved: {other:?}"),
    };
    assert_eq!(file, "quick-check.toml");

    let started = execute(&mut w, Effect::Learn(LearnEffect::Start(file)), NOW_NS);
    match &started[..] {
        [Msg::Learn(LearnMsg::Started(Ok((run, graph))))] => {
            assert_eq!(
                run.plan.effect,
                ScheduleEffect::PracticeOnly,
                "a custom run never changes the schedule"
            );
            assert_eq!(
                graph.name, "Quick check",
                "the run carries the mode it follows"
            );
        }
        other => panic!("the run did not start: {other:?}"),
    }
}

#[test]
fn starting_a_mode_that_is_not_saved_is_refused() {
    let vault = sample_vault("learn-app-missing");
    let mut w = worker_with_host(None, None, Host::default());
    execute(
        &mut w,
        Effect::OpenVault(vault.path.to_string_lossy().into_owned()),
        NOW_NS,
    );
    let out = execute(
        &mut w,
        Effect::Learn(LearnEffect::Start("missing.toml".into())),
        NOW_NS,
    );
    assert!(
        matches!(&out[..], [Msg::Learn(LearnMsg::Started(Err(_)))]),
        "{out:?}"
    );
}
