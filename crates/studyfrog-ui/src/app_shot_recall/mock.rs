//! Screenshot setups for the mock exam: an exam made offline, and a graded one.

use crate::effect::Effect;
use crate::model::{Model, apply};
use crate::msg::Msg;
use crate::state::Screen;

/// A mock exam made offline from a short note, shown with its answers hidden.
pub fn mock_demo(model: &mut Model) {
    let notes = "# Reliabilität\nReliabilität: consistency of measurement over repeated testing occasions.\n\nThe reliability coefficient compares true score variance with observed score variance.\n\n# Normierung\nStandard scores place a result against the norm group mean and standard deviation.\n";
    let chunks = studyfrog_rehearsal::curriculum::chunk_markdown(notes);
    let g = studyfrog_service::exam_gen::generate(&chunks, 6, "Mock exam: bio", None);
    model.state.screen = Screen::Rehearse;
    model.state.rehearse.mock = Some(crate::setup::mock_exam::MockExamUi {
        folder: Some("bio".to_owned()),
        title: "Mock exam: bio".to_owned(),
        result: Some(Box::new(g)),
        ..Default::default()
    });
}

/// A graded mock exam: the first answer complete, the others empty.
pub fn mock_graded(model: &mut Model) {
    mock_demo(model);
    let Some(ui) = model.state.rehearse.mock.as_mut() else {
        return;
    };
    let Some(g) = ui.result.as_ref() else { return };
    let mut answers = vec![String::new(); g.keys.len()];
    if let Some(k) = g.keys.first() {
        answers[0] = format!("{} {}", k.sample_answer, k.rubric.join(" "));
    }
    ui.graded = Some(crate::setup::mock_exam::grading::grade(
        &g.exam, &g.keys, &answers,
    ));
    ui.answers = answers;
}

/// A rehearsal item with a long answer, flipped so it awaits its rating: the item's area scrolls and the
/// rating buttons stay under it (`--screenshot run-long`).
pub fn long_run(model: &mut Model) -> Vec<Effect> {
    use studyfrog_core::plan::plan_spaced;
    use studyfrog_core::run::{Run, RunMode, RunState};
    use studyfrog_core::uid::Uid;
    use studyfrog_rehearsal::input::Action;
    use studyfrog_service::content::ItemContent;
    let run = Run {
        uid: Uid(100),
        mode: RunMode::Flip,
        plan: plan_spaced(),
        queue: vec![Uid(1)],
        state: RunState::Idle,
        started_at: None,
    };
    model.state.now = 0.0;
    model.state.screen = Screen::Rehearse;
    let _ = apply(model, Msg::RunReady(Ok(run)));
    let answer = (1..=30)
        .map(|i| format!("Paragraph {i}: the answer goes on for a while, with enough words to wrap across the card, so the item has more text than fits."))
        .collect::<Vec<_>>()
        .join("\n\n");
    let content = ItemContent {
        case: None,
        title: "Long item".into(),
        prompt: "Explain the reliability coefficient in full.".into(),
        answer,
        previous: None,
        source: None,
        color: Some("teal".into()),
        expected: None,
        occlusion: None,
        is_note: false,
        comments: Vec::new(),
    };
    let _ = apply(model, Msg::ItemLoaded(Uid(1), Ok(content)));
    // The demo's own messages are applied here; no effect is left to load an item from a vault.
    let _ = apply(model, Msg::RunAction(Action::Flip));
    Vec::new()
}

/// The learn-mode editor with a mode open and one point left to fix (`--screenshot learn-editor`).
pub fn learn_editor_demo(model: &mut Model) {
    use studyfrog_rehearsal::learn_graph::{Edge, LearnGraph, Node, NodeKind};
    let node = |id: &str, kind, label: &str| Node {
        id: id.into(),
        kind,
        label: label.into(),
    };
    let graph = LearnGraph {
        name: "Interleave two decks".into(),
        nodes: vec![
            node("n1", NodeKind::Start, "start"),
            node("n2", NodeKind::Present, "show a card from deck A or B"),
            node("n3", NodeKind::Rate, "rate"),
            node("n4", NodeKind::Branch, "pick the next deck"),
            node("n5", NodeKind::Stop, "end"),
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
            Edge {
                from: "n3".into(),
                to: "n4".into(),
                when: None,
            },
            Edge {
                from: "n4".into(),
                to: "n2".into(),
                when: Some("again".into()),
            },
        ],
    };
    let mut e = crate::learn_editor::new_editor();
    e.modes = vec![
        "interleave-two-decks.toml".into(),
        "spacing-test.toml".into(),
    ];
    e.file = Some("interleave-two-decks.toml".into());
    e.graph = graph;
    e.reasons = vec!["the branch n4 needs a default way on".into()];
    e.report = Some(studyfrog_store::learn_report::ModeReport {
        mode_id: "interleave-two-decks".into(),
        runs: 4,
        finished: 3,
        items: 27,
        visits: vec![("n2".into(), 61), ("n3".into(), 61), ("n4".into(), 61)],
        ratings: vec![
            ("again".into(), 14),
            ("easy".into(), 12),
            ("good".into(), 25),
            ("hard".into(), 10),
        ],
        branches: vec![("again".into(), 14), ("default".into(), 47)],
    });
    model.state.screen = Screen::Rehearse;
    model.state.modal = Some(crate::modal::Modal::LearnEditor(Box::new(e)));
}

/// The job editor with three jobs: one on a schedule, one on an event, one switched off (`--screenshot script-jobs`).
pub fn script_jobs_demo(model: &mut Model) {
    use studyfrog_service::scripting::config::Access;
    use studyfrog_service::scripting::jobs::JobRow;
    let row = |name: &str, schedule: &str, on: &[&str], enabled: bool, next: Option<&str>| JobRow {
        name: name.into(),
        script: format!("{name}.py"),
        args: Vec::new(),
        schedule: schedule.into(),
        on: on.iter().map(|s| (*s).to_owned()).collect(),
        timeout_secs: 60,
        access: Access::Read,
        enabled,
        next_run: next.map(str::to_owned),
    };
    let e = crate::console_ui::jobs::JobsEditor {
        rows: vec![
            row(
                "morning-backup",
                "0 8 * * mon-fri",
                &[],
                true,
                Some("2027-03-01 08:00"),
            ),
            row("tag-new-cards", "", &["item_created"], true, None),
            row("weekly-report", "0 9 * * 1", &[], false, None),
        ],
        errors: vec!["job 4: unknown event vault_closing".into()],
        python: "python3".into(),
        scripts: vec!["morning-backup.py".into(), "tag-new-cards.py".into()],
        selected: Some(0),
        history: vec!["[12:00] script morning-backup finished".into()],
        adhoc_script: "morning-backup.py".into(),
        adhoc_args: String::new(),
        new_name: String::new(),
        new_script: String::new(),
        dirty: true,
        notice: None,
    };
    model.state.screen = Screen::Console;
    model.state.modal = Some(crate::modal::Modal::ScriptJobs(Box::new(e)));
}
