#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Stress tests for details notebook tab taxonomy and switching invariants.

mod common;

use common::views::{harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::event::Difficulty;
use studyfrog_core::priority::Priority;
use studyfrog_core::uid::Uid;
use studyfrog_md::parse::parse_doc;
use studyfrog_service::document::LoadedDoc;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;

const CARD_RAW: &str = "\
---
id: 0000000000000000000000000B
kind: card
title: Vocabulary
---
Bonjour
???
Hello
";

const QUESTION_RAW: &str = "\
---
id: 0000000000000000000000000C
kind: question
title: Quiz Question
---
What is 2+2?
???
4
";

const STACK_RAW: &str = "\
---
id: 00000000000000000000000001
kind: stack
title: Math Stack
---
# Math
";

fn load_item(m: &mut Model, uid: Uid, path: &str, raw: &str) {
    let doc = parse_doc(raw).unwrap();
    let ld = LoadedDoc {
        path: path.into(),
        doc,
        hash: 1,
    };
    apply(m, Msg::DocumentLoaded(uid, Ok(ld)));
}

#[test]
fn tab_invariants_strictly_enforced_across_all_kinds() {
    let mut m = open_model();
    load_item(&mut m, Uid(1), "math/_stack.md", STACK_RAW);
    let mut h = harness(m);
    h.run_steps(2);

    for tab in ["Properties", "Schedule", "Relations"] {
        assert!(h.query_by_label(tab).is_some(), "Stack missing {tab}");
    }
    assert!(h.query_by_label("Learning").is_none(), "Stack has Learning");

    load_item(&mut h.state_mut().0, Uid(0xB), "math/vocab.md", CARD_RAW);
    h.run_steps(2);
    for tab in ["Properties", "Relations", "Learning"] {
        assert!(h.query_by_label(tab).is_some(), "Card missing {tab}");
    }
    assert!(h.query_by_label("Schedule").is_none(), "Card has Schedule");

    load_item(&mut h.state_mut().0, Uid(0xC), "math/quiz.md", QUESTION_RAW);
    h.run_steps(2);
    for tab in ["Properties", "Relations", "Learning"] {
        assert!(h.query_by_label(tab).is_some(), "Question missing {tab}");
    }
    assert!(
        h.query_by_label("Schedule").is_none(),
        "Question has Schedule"
    );
}

#[test]
fn tab_selection_sanitization_on_item_switch() {
    let mut m = open_model();
    load_item(&mut m, Uid(1), "math/_stack.md", STACK_RAW);
    let mut h = harness(m);
    h.run_steps(2);

    h.get_by_label("Schedule").click();
    h.run_steps(2);
    assert!(h.query_by_label("No due date set").is_some());

    load_item(&mut h.state_mut().0, Uid(0xB), "math/vocab.md", CARD_RAW);
    h.run_steps(2);
    assert!(h.query_by_label("Schedule").is_none());
    assert!(h.query_by_label("Tags (comma separated)").is_some());

    h.get_by_label("Learning").click();
    h.run_steps(2);
    assert!(h.query_by_label("Familiarity").is_some());

    load_item(&mut h.state_mut().0, Uid(1), "math/_stack.md", STACK_RAW);
    h.run_steps(2);
    assert!(h.query_by_label("Learning").is_none());
    assert!(h.query_by_label("Tags (comma separated)").is_some());

    h.get_by_label("Relations").click();
    h.run_steps(2);
    assert!(h.query_by_label("Links to").is_some());

    load_item(&mut h.state_mut().0, Uid(0xB), "math/vocab.md", CARD_RAW);
    h.run_steps(2);
    assert!(h.query_by_label("Links to").is_some());
}

#[test]
fn property_mutations_and_difficulty_scoping() {
    let mut m = open_model();
    load_item(&mut m, Uid(1), "math/_stack.md", STACK_RAW);
    let mut h = harness(m);
    h.run_steps(2);

    assert!(h.query_by_label("Difficulty: inherit").is_none());
    h.get_by_label("Priority: low").click();
    h.run_steps(2);
    let p = h.state().0.state.editor.as_ref().unwrap().form.priority;
    assert_eq!(p, Some(Priority::Low));

    h.get_by_label("Tags (comma separated)").focus();
    h.get_by_label("Tags (comma separated)")
        .type_text("algebra, basis");
    h.run_steps(2);
    let t = &h.state().0.state.editor.as_ref().unwrap().form.tags;
    assert_eq!(t, "algebra, basis");

    load_item(&mut h.state_mut().0, Uid(0xB), "math/vocab.md", CARD_RAW);
    h.run_steps(2);
    assert!(h.query_by_label("Difficulty: inherit").is_some());
    h.get_by_label("Difficulty: medium").click();
    h.run_steps(2);
    let d = h.state().0.state.editor.as_ref().unwrap().form.difficulty;
    assert_eq!(d, Some(Difficulty::Medium));
}
