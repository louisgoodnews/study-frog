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
    model.state.screen = Screen::Rehearse;
    model.state.modal = Some(crate::modal::Modal::LearnEditor(Box::new(e)));
}
