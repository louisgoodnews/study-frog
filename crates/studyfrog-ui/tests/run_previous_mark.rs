#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! STORY-0196: the rating button that matches an item's previous rating shows the difficulty it stood for,
//! in its colour and with its word; an item with no previous rating shows no mark.

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
        comments: Vec::new(),
    }
}

#[test]
fn the_previous_rating_shows_its_difficulty_on_the_matching_button() {
    let mut m = running_model(RunMode::Flip);
    apply(
        &mut m,
        Msg::ItemLoaded(Uid(1), Ok(content(Some(Rating::Hard)))),
    );
    let mut h = harness(m);
    h.set_size(eframe::egui::vec2(1280.0, 800.0));
    h.run();
    h.get_by_label("Show answer (Space)").click();
    h.run();
    assert!(
        h.query_all_by_value("Last time: Medium").next().is_some(),
        "the marked word shows"
    );
    assert!(h.query_by_value("Last time: Difficult").is_none());
    assert!(h.query_by_value("Last time: Easy").is_none());
}

#[test]
fn an_item_without_a_previous_rating_shows_no_mark() {
    let mut m = running_model(RunMode::Flip);
    apply(&mut m, Msg::ItemLoaded(Uid(1), Ok(content(None))));
    let mut h = harness(m);
    h.set_size(eframe::egui::vec2(1280.0, 800.0));
    h.run();
    h.get_by_label("Show answer (Space)").click();
    h.run();
    assert!(h.query_by_value("Last time: Medium").is_none());
}
