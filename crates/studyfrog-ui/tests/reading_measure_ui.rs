#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! STORY-0114 drawn: prose on a help page and in a rehearsal card never runs wider than the reading measure.

mod common;

use common::views::{harness, open_model, running_model};
use eframe::egui::FontId;
use egui_kittest::kittest::Queryable;
use studyfrog_core::run::RunMode;
use studyfrog_core::uid::Uid;
use studyfrog_service::content::ItemContent;
use studyfrog_service::help::PageRef;
use studyfrog_ui::help_types::HelpMsg;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::typo::{MEASURE_CHARS, char_width};

const LONG: &str = "Because small, uncharged molecules such as oxygen and carbon dioxide slip through the lipids \
by simple diffusion, while ions and large molecules need a channel or a pump, the membrane is called selectively \
permeable, and active transport spends energy to move substances against their gradient.";

fn measure_at(ctx: &eframe::egui::Context, size: f32) -> f32 {
    (char_width(ctx, &FontId::proportional(size)) * MEASURE_CHARS).round()
}

#[test]
fn help_paragraphs_keep_to_the_measure_on_a_wide_window() {
    let mut m = open_model();
    m.state.toasts.clear();
    let mut h = harness(m);
    h.set_size(eframe::egui::vec2(1600.0, 900.0));
    apply(
        &mut h.state_mut().0,
        Msg::Help(HelpMsg::Open(PageRef::Supplied("getting-started".into()))),
    );
    h.run();
    let measure = measure_at(&h.ctx, 15.0);
    let para = h
        .get_by_label_contains("This page takes you from an empty folder")
        .rect();
    assert!(
        para.width() <= measure + 1.0,
        "{} > {measure}",
        para.width()
    );
    assert!(
        para.width() > measure * 0.8,
        "the paragraph fills the measure: {}",
        para.width()
    );
}

#[test]
fn a_long_answer_in_a_run_keeps_to_the_measure() {
    let mut m = running_model(RunMode::Flip);
    let content = ItemContent {
        title: "Membrane".into(),
        prompt: "Why is the membrane selectively permeable?".into(),
        answer: LONG.into(),
        previous: None,
        source: None,
        color: None,
        expected: None,
        occlusion: None,
    };
    apply(&mut m, Msg::ItemLoaded(Uid(1), Ok(content)));
    let mut h = harness(m);
    h.set_size(eframe::egui::vec2(1600.0, 900.0));
    h.run();
    h.get_by_label("Show answer (Space)").click();
    h.run();
    let measure = measure_at(&h.ctx, 15.0);
    let answer = h.get_by_label_contains("Because small, uncharged").rect();
    assert!(
        answer.width() <= measure + 1.0,
        "{} > {measure}",
        answer.width()
    );
}
