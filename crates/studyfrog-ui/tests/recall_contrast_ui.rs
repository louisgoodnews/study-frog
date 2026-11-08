#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The contrastive drill (STORY-0111): opened from a confused pair or 2-4 selected items, matched, compared
//! side by side and kept as hints or a comparison note.

mod common;

use common::analytics::analytics;
use common::flow::send;
use common::reports::bundle;
use common::views::{assert_all_named, harness, lib_item, open_model};
use common::{run_to_idle, temp_dir};
use egui_kittest::kittest::Queryable;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_service::contrast::{ContrastItem, ContrastTarget};
use studyfrog_ui::contrast::{ContrastMsg, msg, right, shuffled};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::worker_new;
use studyfrog_ui::explorer_ui::ReportTab;
use studyfrog_ui::library_bulk::{BulkMsg, LibraryEffect};
use studyfrog_ui::library_ui::LibraryMsg;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::recall_types::RecallEffect;
use studyfrog_ui::state::Screen;

fn item(n: u128, title: &str, prompt: &str, answer: &str) -> ContrastItem {
    ContrastItem {
        uid: Uid(n),
        title: title.into(),
        prompt: prompt.into(),
        answer: answer.into(),
    }
}

fn loaded() -> Model {
    let mut m = open_model();
    apply(&mut m, msg(ContrastMsg::Open(vec![Uid(10), Uid(11)])));
    let items = vec![
        item(
            10,
            "Ionic bond",
            "What is an ionic bond?",
            "electrons are transferred",
        ),
        item(
            11,
            "Covalent bond",
            "What is a covalent bond?",
            "electrons are shared",
        ),
    ];
    apply(&mut m, msg(ContrastMsg::Loaded(Ok(items))));
    m
}

#[test]
fn a_confused_pair_in_the_reports_opens_the_drill() {
    let mut m = open_model();
    m.state.screen = Screen::Reports;
    m.state.explorer.tab = ReportTab::Leeches;
    let mut b = bundle();
    b.analytics = Box::new(analytics());
    m.state.reports = Some(b);
    let mut h = harness(m);
    h.run_steps(3);
    h.get_by_label("Compare Ionic bond and Covalent bond side by side")
        .click();
    h.run_steps(2);
    let c = h.state().0.state.recall.contrast.clone().expect("open");
    assert_eq!(c.uids, [Uid(10), Uid(11)]);
    assert!(c.busy, "loading");
}

#[test]
fn two_to_four_selected_items_offer_the_drill_in_the_library() {
    let mut m = open_model();
    let rows: Vec<_> = (1..=5).map(|i| lib_item(i, &format!("Item {i}"))).collect();
    apply(&mut m, Msg::Library(LibraryMsg::Loaded(Ok(rows))));
    apply(
        &mut m,
        Msg::Library(LibraryMsg::Bulk(BulkMsg::Select(Uid(1), true))),
    );
    let mut h = harness(m);
    h.run_steps(3);
    assert!(
        h.query_by_label("Contrastive drill").is_none(),
        "one item is not enough"
    );
    let mut m = std::mem::take(&mut h.state_mut().0);
    apply(
        &mut m,
        Msg::Library(LibraryMsg::Bulk(BulkMsg::Select(Uid(2), true))),
    );
    let mut h = harness(m);
    h.run_steps(3);
    h.get_by_label("Contrastive drill").click();
    h.run_steps(2);
    assert!(h.state().0.state.recall.contrast.is_some());
    let fx = std::mem::take(&mut h.state_mut().1);
    let load = Effect::Library(LibraryEffect::Recall(RecallEffect::LoadContrast(vec![
        Uid(1),
        Uid(2),
    ])));
    assert!(fx.contains(&load), "{fx:?}");
    let mut m = open_model();
    assert!(apply(&mut m, msg(ContrastMsg::Open(vec![Uid(1)]))).is_empty());
    assert!(m.state.notice.is_some(), "too few items are refused");
}

#[test]
fn the_shuffle_never_leaves_an_answer_under_its_own_prompt() {
    for n in 2..=4 {
        for seed in 0..200 {
            let order = shuffled(n, seed);
            let mut sorted = order.clone();
            sorted.sort_unstable();
            assert_eq!(sorted, (0..n).collect::<Vec<_>>());
            assert!(order.iter().enumerate().all(|(k, i)| k != *i), "{order:?}");
        }
    }
}

#[test]
fn matching_checking_comparing_and_keeping_the_distinction() {
    let mut h = harness(loaded());
    h.run_steps(3);
    for label in [
        "What is an ionic bond?",
        "What is a covalent bond?",
        "electrons are shared",
    ] {
        assert!(h.query_all_by_label(label).count() >= 1, "missing {label}");
    }
    assert_all_named(&h, "contrastive drill");
    let order = h
        .state()
        .0
        .state
        .recall
        .contrast
        .as_ref()
        .unwrap()
        .order
        .clone();
    let slot_of = |item: usize| ["A", "B"][order.iter().position(|i| *i == item).unwrap()];
    h.get_by_label(&format!("Answer {} for What is an ionic bond?", slot_of(0)))
        .click();
    h.get_by_label(&format!(
        "Answer {} for What is a covalent bond?",
        slot_of(0)
    ))
    .click();
    h.run_steps(2);
    h.get_by_label("Check the matches").click();
    h.run_steps(3);
    let c = h.state().0.state.recall.contrast.clone().unwrap();
    assert_eq!(right(&c), [true, false]);
    for label in [
        "Right",
        "Not this one",
        "Only here: transferred",
        "Only here: shared",
    ] {
        assert!(h.query_by_label(label).is_some(), "missing {label}");
    }
    h.get_by_label("What is the key difference that distinguishes them?")
        .focus();
    h.get_by_label("What is the key difference that distinguishes them?")
        .type_text("given or shared");
    h.get_by_label("Create or update a comparison note").click();
    h.run_steps(2);
    h.get_by_label("Keep the distinction").click();
    h.run_steps(2);
    let fx = std::mem::take(&mut h.state_mut().1);
    let saved = fx.iter().any(|e| matches!(e,
        Effect::Library(LibraryEffect::Recall(RecallEffect::SaveContrast(s)))
            if s.text == "given or shared" && s.target == ContrastTarget::Note && s.uids == [Uid(10), Uid(11)]));
    assert!(saved, "{fx:?}");
}

#[test]
fn the_worker_loads_the_items_and_writes_the_hints() {
    let dir = temp_dir("contrast-flow");
    let bio = dir.path.join("bio");
    std::fs::create_dir_all(&bio).unwrap();
    let id = |n: u128| format_ulid(Uid(n));
    std::fs::write(
        bio.join("_stack.md"),
        format!("---\nid: {}\nkind: stack\ntitle: Bio\n---\n", id(1)),
    )
    .unwrap();
    for (n, name, back) in [(10, "Ionic", "transferred"), (11, "Covalent", "shared")] {
        let text = format!(
            "---\nid: {}\nkind: card\ntitle: {name}\n---\n{name}?\n???\n{back}\n",
            id(n)
        );
        std::fs::write(bio.join(format!("{name}.md")), text).unwrap();
    }
    let mut w = worker_new(None);
    let mut m = Model::default();
    m.state.app.tutorial_offered = true;
    run_to_idle(
        &mut m,
        &mut w,
        vec![Effect::OpenVault(dir.path.display().to_string())],
    );
    send(
        &mut m,
        &mut w,
        msg(ContrastMsg::Open(vec![Uid(10), Uid(11)])),
    );
    assert_eq!(m.state.recall.contrast.as_ref().unwrap().items.len(), 2);
    send(
        &mut m,
        &mut w,
        msg(ContrastMsg::Distinction("given or shared".into())),
    );
    send(&mut m, &mut w, msg(ContrastMsg::Save));
    let c = m.state.recall.contrast.as_ref().unwrap();
    assert!(c.saved.is_some(), "{:?}", c.error);
    let ionic = std::fs::read_to_string(bio.join("Ionic.md")).unwrap();
    assert!(
        ionic.contains("> Distinction (vs [[Covalent]]): given or shared"),
        "{ionic}"
    );
    assert!(m.state.library.undo.is_some());
}
