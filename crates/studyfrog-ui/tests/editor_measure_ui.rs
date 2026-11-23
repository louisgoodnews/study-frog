#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! STORY-0114 in the note editor: the text column keeps to the reading measure, and the details stay beside it.

mod common;

use common::views::{harness, open_model};
use eframe::egui::{self, FontId};
use egui_kittest::kittest::Queryable;
use studyfrog_core::uid::Uid;
use studyfrog_md::parse::parse_doc;
use studyfrog_service::document::LoadedDoc;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;
use studyfrog_ui::typo::{MEASURE_CHARS, char_width};

const NOTE: &str = "---\nid: 0000000000000000000000000A\nkind: note\ntitle: Zellmembran\n---\n\
Die Zellmembran ist eine dünne, flexible Barriere, die das Innere jeder Zelle von der Außenwelt trennt. \
Kleine, ungeladene Moleküle diffundieren ungehindert durch die Lipiddoppelschicht.\n\n\
Deshalb nennt man die Membran selektiv permeabel. Der aktive Transport verbraucht Energie.\n";

fn measure_at(ctx: &egui::Context, size: f32) -> f32 {
    (char_width(ctx, &FontId::proportional(size)) * MEASURE_CHARS).round()
}

fn editing() -> Model {
    let mut m = open_model();
    let doc = parse_doc(NOTE).unwrap();
    apply(
        &mut m,
        Msg::DocumentLoaded(
            Uid(0xA),
            Ok(LoadedDoc {
                path: "lernen/zellmembran.md".into(),
                doc,
                hash: 1,
            }),
        ),
    );
    assert_eq!(m.state.screen, Screen::Editor);
    m
}

/// The body's text area at `width` x `height`, in Write mode or in the default Split mode.
fn check_at(width: f32, height: f32, write: bool) {
    let mut h = harness(editing());
    h.set_size(egui::vec2(width, height));
    h.run_steps(3);
    if write {
        h.get_by_label("Body: write").click();
        h.run_steps(3);
    }
    let measure = measure_at(&h.ctx, 15.0);
    let body = h
        .get_by_role_and_label(egui::accesskit::Role::MultilineTextInput, "Body")
        .rect();
    assert!(
        body.width() <= measure + 1.0,
        "at {width}x{height} (write: {write}) the text column is {} wide, over the measure {measure}",
        body.width()
    );
    assert!(
        h.query_by_label("Details").is_some(),
        "the details are beside the text at {width}x{height}"
    );
    assert!(h.query_by_label("Properties").is_some());
}

#[test]
fn the_text_column_keeps_to_the_measure_at_1280_by_800() {
    check_at(1280.0, 800.0, true);
    check_at(1280.0, 800.0, false);
}

#[test]
fn the_text_column_keeps_to_the_measure_at_1600_by_900() {
    check_at(1600.0, 900.0, true);
    check_at(1600.0, 900.0, false);
}
