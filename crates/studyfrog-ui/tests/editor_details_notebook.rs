#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Integration tests for the details view tabbed notebook and stack due date editing.

mod common;

use common::views::{harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_calendar::session::EventKind;
use studyfrog_core::civil::Date;
use studyfrog_core::event::Difficulty;
use studyfrog_core::priority::Priority;
use studyfrog_core::uid::Uid;
use studyfrog_md::parse::parse_doc;
use studyfrog_service::document::LoadedDoc;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::plan_msg::PlanMsg;
use studyfrog_ui::state::Screen;

const NOTE: &str = "\
---
id: 0000000000000000000000000A
kind: note
title: Alpha
---
# Heading

Content.
";

const STACK: &str = "\
---
id: 00000000000000000000000001
kind: stack
title: French
---
# French Stack
";

fn editing_note() -> Model {
    let mut m = open_model();
    let doc = parse_doc(NOTE).unwrap();
    apply(
        &mut m,
        Msg::DocumentLoaded(
            Uid(0xA),
            Ok(LoadedDoc {
                path: "french/alpha.md".into(),
                doc,
                hash: 1,
            }),
        ),
    );
    assert_eq!(m.state.screen, Screen::Editor);
    m
}

fn editing_stack() -> Model {
    let mut m = open_model();
    let doc = parse_doc(STACK).unwrap();
    apply(
        &mut m,
        Msg::DocumentLoaded(
            Uid(0x1),
            Ok(LoadedDoc {
                path: "french/_stack.md".into(),
                doc,
                hash: 2,
            }),
        ),
    );
    assert_eq!(m.state.screen, Screen::Editor);
    m
}

#[test]
fn notebook_tabs_switching_for_content_items() {
    let mut h = harness(editing_note());
    h.run_steps(2);

    for tab in ["Properties", "Relations", "Learning"] {
        assert!(h.query_by_label(tab).is_some(), "missing tab {tab}");
    }
    assert!(
        h.query_by_label("Schedule").is_none(),
        "Schedule tab should only be for stacks"
    );

    assert!(h.query_by_label("Kind").is_some());
    assert!(h.query_by_label("Tags (comma separated)").is_some());

    h.get_by_label("Relations").click();
    h.run_steps(2);
    assert!(h.query_by_label("Links to").is_some());

    h.get_by_label("Learning").click();
    h.run_steps(2);
    assert!(h.query_by_label("Familiarity").is_some());

    h.get_by_label("Properties").click();
    h.run_steps(2);
    assert!(h.query_by_label("Tags (comma separated)").is_some());
}

#[test]
fn mutable_fields_in_properties_tab() {
    let mut h = harness(editing_note());
    h.run_steps(2);

    h.get_by_label("Tags (comma separated)").focus();
    h.get_by_label("Tags (comma separated)")
        .type_text("grammar, verbs");
    h.run_steps(2);
    let tags = &h.state().0.state.editor.as_ref().unwrap().form.tags;
    assert_eq!(tags, "grammar, verbs");

    h.get_by_label("Priority: high").click();
    h.run_steps(2);
    assert_eq!(
        h.state().0.state.editor.as_ref().unwrap().form.priority,
        Some(Priority::High)
    );

    h.get_by_label("Difficulty: easy").click();
    h.run_steps(2);
    assert_eq!(
        h.state().0.state.editor.as_ref().unwrap().form.difficulty,
        Some(Difficulty::Easy)
    );
}

#[test]
fn stack_notebook_tabs_and_inline_due_date_editing() {
    let m = editing_stack();
    let mut h = harness(m);
    h.run_steps(2);

    for tab in ["Properties", "Schedule", "Relations"] {
        assert!(h.query_by_label(tab).is_some(), "missing stack tab {tab}");
    }
    assert!(
        h.query_by_label("Learning").is_none(),
        "Learning tab should not be for stacks"
    );

    h.get_by_label("Schedule").click();
    h.run_steps(2);
    assert!(h.query_by_label("No due date set").is_some());

    h.get_by_label("Set due in 1 week").click();
    h.run_steps(2);

    let saved_session = h
        .state()
        .1
        .iter()
        .find_map(|e| match e {
            Effect::SaveSession(s) => Some(s.clone()),
            _ => None,
        })
        .expect("expected Effect::SaveSession when setting stack due date");

    assert_eq!(saved_session.kind, EventKind::Deadline);
    assert_eq!(saved_session.stack, Some(Uid(1)));

    apply(
        &mut h.state_mut().0,
        Msg::Plan(PlanMsg::Changed(Ok(vec![saved_session.clone()]))),
    );
    h.state_mut().1.clear();
    h.run_steps(2);

    assert!(h.query_by_label("Current due date:").is_some());
    assert!(h.query_by_label("Clear due date").is_some());
    assert!(h.query_by_label("Remind me:").is_some());

    h.get_by_label("Clear due date").click();
    h.run_steps(2);

    let deleted_id = h
        .state()
        .1
        .iter()
        .find_map(|e| match e {
            Effect::DeleteSession(id) => Some(*id),
            _ => None,
        })
        .expect("expected Effect::DeleteSession when clearing stack due date");

    assert_eq!(deleted_id, saved_session.id);
}

#[test]
fn stack_due_date_text_field_editing() {
    let m = editing_stack();
    let mut h = harness(m);
    h.run_steps(2);

    h.get_by_label("Schedule").click();
    h.run_steps(2);

    h.get_by_label("Due date").focus();
    h.get_by_label("Due date").type_text("2026-12-31");
    h.run_steps(2);

    let saved = h
        .state()
        .1
        .iter()
        .find_map(|e| match e {
            Effect::SaveSession(s) => Some(s.clone()),
            _ => None,
        })
        .expect("expected SaveSession after typing date");

    assert_eq!(
        saved.start.date,
        Date {
            year: 2026,
            month: 12,
            day: 31
        }
    );
}
