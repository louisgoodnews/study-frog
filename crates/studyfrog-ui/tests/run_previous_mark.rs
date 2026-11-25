#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! STORY-0196: the rating button that matches an item's previous rating shows the difficulty it stood for,
//! in its colour and with its word; an item with no previous rating shows no mark.
use studyfrog_ui::msg::MsgRehearsal;

mod common;

use common::views::{harness, running_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::review::Rating;
use studyfrog_core::run::RunMode;
use studyfrog_core::uid::Uid;
use studyfrog_service::content::ItemContent;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;

fn content(previous: Option<Rating>) -> ItemContent {
    ItemContent {
        case: None,
        title: "Membrane".into(),
        prompt: "Why is the membrane selectively permeable?".into(),
        answer: "Because small molecules pass the lipids.".into(),
        previous,
        source: None,
        color: None,
        expected: None,
        occlusion: None,
        is_note: false,
        answer_type: None,
        comments: Vec::new(),
    }
}

#[test]
fn the_previous_rating_shows_its_difficulty_on_the_matching_button() {
    let mut m = running_model(RunMode::Flip);
    apply(
        &mut m,
        Msg::Rehearsal(MsgRehearsal::ItemLoaded(
            Uid(1),
            Ok(content(Some(Rating::Hard))),
        )),
    );
    let mut h = harness(m);
    h.set_size(eframe::egui::vec2(1280.0, 800.0));
    h.run();
    h.get_by_label("Show answer (Space)").click();
    h.run();
    assert!(
        h.query_all_by_value("Last: Medium").next().is_some(),
        "the marked word shows"
    );
    assert!(h.query_by_value("Last: Difficult").is_none());
    assert!(h.query_by_value("Last: Easy").is_none());
}

#[test]
fn an_item_without_a_previous_rating_shows_no_mark() {
    let mut m = running_model(RunMode::Flip);
    apply(
        &mut m,
        Msg::Rehearsal(MsgRehearsal::ItemLoaded(Uid(1), Ok(content(None)))),
    );
    let mut h = harness(m);
    h.set_size(eframe::egui::vec2(1280.0, 800.0));
    h.run();
    h.get_by_label("Show answer (Space)").click();
    h.run();
    assert!(h.query_by_value("Last: Medium").is_none());
}

/// STORY-0196 layout: the "Last" line does not move the rating buttons out of line, and the hint line
/// stays on the screen at 1280 by 800 (the card takes the room the line needs).
#[test]
fn the_previous_mark_keeps_the_buttons_level_and_the_hints_on_screen() {
    let mut m = running_model(RunMode::Flip);
    apply(
        &mut m,
        Msg::Rehearsal(MsgRehearsal::ItemLoaded(
            Uid(1),
            Ok(content(Some(Rating::Good))),
        )),
    );
    let mut h = harness(m);
    h.set_size(eframe::egui::vec2(1280.0, 800.0));
    h.run();
    h.get_by_label("Show answer (Space)").click();
    h.run();
    let again = h.get_by_label("Again (1)").rect().top();
    let good = h.get_by_label("Good (3)").rect().top();
    assert!(
        (again - good).abs() < 0.5,
        "buttons level: {again} vs {good}"
    );
    let hints = h
        .query_by_label_contains("Space shows the answer")
        .expect("hint line")
        .rect();
    assert!(
        hints.bottom() <= 800.0,
        "hints on screen: {}",
        hints.bottom()
    );
}
