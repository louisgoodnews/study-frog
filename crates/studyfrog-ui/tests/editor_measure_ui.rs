#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! STORY-0114 in the note editor: the text column keeps to the reading measure, and the details stay beside it.
use studyfrog_ui::msg::MsgEditor;

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
use studyfrog_ui::typo::{self, MEASURE_CHARS, char_width};

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
        Msg::Editor(MsgEditor::DocumentLoaded(
            Uid(0xA),
            Ok(LoadedDoc {
                path: "lernen/zellmembran.md".into(),
                doc,
                hash: 1,
            }),
        )),
    );
    assert_eq!(m.state.screen, Screen::Editor);
    m
}

/// The body's text area at `width` x `height`, in Write mode or in Split mode (the rendering under the text).
fn check_at(width: f32, height: f32, write: bool) {
    let mut h = harness(editing());
    h.set_size(egui::vec2(width, height));
    h.run_steps(3);
    let mode = if write {
        "Body: write"
    } else {
        "Body: preview below"
    };
    h.get_by_label(mode).click();
    h.run_steps(3);
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

/// The mean characters per full line of `prose` in the editor's body font, wrapped at `width`.
fn mean_chars_per_line(ctx: &egui::Context, width: f32) -> f32 {
    let font = FontId::proportional(15.0);
    let galley =
        ctx.fonts_mut(|f| f.layout(typo::SAMPLE.repeat(6), font, egui::Color32::WHITE, width));
    let rows = &galley.rows;
    let counts: Vec<usize> = rows[..rows.len() - 1]
        .iter()
        .map(|r| r.char_count_including_newline().0)
        .collect();
    counts.iter().sum::<usize>() as f32 / counts.len() as f32
}

/// A note's body holds 55 to 75 characters per line in Write and in Split (the rendering under the text), at
/// 1280 by 800 and 960 by 640.
#[test]
fn a_note_body_holds_55_to_75_characters_per_line_in_write_and_split() {
    for (width, height) in [(1280.0, 800.0), (960.0, 640.0)] {
        for label in ["Body: write", "Body: preview below"] {
            let mut h = harness(editing());
            h.set_size(egui::vec2(width, height));
            h.run_steps(3);
            h.get_by_label(label).click();
            h.run_steps(3);
            let body = h
                .get_by_role_and_label(egui::accesskit::Role::MultilineTextInput, "Body")
                .rect()
                .width();
            let mean = mean_chars_per_line(&h.ctx, body);
            assert!(
                (55.0..=75.0).contains(&mean),
                "at {width}x{height} ({label}) the body is {body} wide: {mean} characters per line"
            );
        }
    }
}
