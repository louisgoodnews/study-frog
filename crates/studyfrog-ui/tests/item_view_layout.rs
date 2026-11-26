#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Headless layout tests for Item view cut-offs (BUG-0059) and horizontal scrollability (STORY-0243).

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::{NodeT, Queryable};
use studyfrog_core::meta::Meta;
use studyfrog_core::uid::Uid;
use studyfrog_md::doc::{CardDoc, Doc};
use studyfrog_service::document::LoadedDoc;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::{Msg, MsgEditor, MsgVault};
use studyfrog_ui::state::Screen;

const LONG_TITLE: &str = "Was versteht man unter dem nomothetischen Ansatz in der Diagnostik?";

fn card_editor_model() -> Model {
    let mut m = open_model();
    apply(&mut m, Msg::Vault(MsgVault::Navigate(Screen::Editor)));
    let doc = Doc::Card(CardDoc {
        meta: Meta {
            uid: Uid(42),
            title: LONG_TITLE.into(),
            tags: vec!["nomothetisch".into()],
            aliases: Vec::new(),
            created: None,
            updated: None,
        },
        front: LONG_TITLE.into(),
        back: "Die Ausrichtung auf allgemeingueltige Gesetze und interindividuelle Unterschiede."
            .into(),
        reversible: false,
        extra: Vec::new(),
    });
    let loaded = LoadedDoc {
        path: "psychologische-diagnostik-und-evaluation/einfuehrung/was-versteht-man-unter-dem-nomothetischen-ansatz-in-der-diag.md".into(),
        doc,
        hash: 1,
    };
    apply(
        &mut m,
        Msg::Editor(MsgEditor::DocumentLoaded(Uid(42), Ok(loaded))),
    );
    m
}

fn width_of(h: &egui_kittest::Harness<'_, common::views::App>, label: &str) -> f32 {
    let b = h
        .get_by_label(label)
        .accesskit_node()
        .bounding_box()
        .unwrap();
    (b.x1 - b.x0) as f32
}

#[test]
fn item_view_title_front_and_color_hex_use_available_width_without_clipping() {
    let mut h = harness(card_editor_model());
    h.set_size(eframe::egui::vec2(1280.0, 800.0));
    h.run_steps(6);

    let title_w = width_of(&h, "Title");
    let front_w = width_of(&h, "Front");
    let hex_w = width_of(&h, "Custom color (#RRGGBB)");

    assert!(
        title_w >= 600.0,
        "Title input should span editor column, got {title_w} px"
    );
    assert!(
        front_w >= 600.0,
        "Front editor should fill card width, got {front_w} px"
    );
    assert!(
        hex_w >= 100.0,
        "Color #RRGGBB input should fit 7 chars cleanly, got {hex_w} px"
    );
    assert_all_named(&h, "item view card editor");
}

#[test]
fn item_view_stays_well_formed_at_960x640() {
    let mut h = harness(card_editor_model());
    h.set_size(eframe::egui::vec2(960.0, 640.0));
    h.run_steps(6);

    let title_w = width_of(&h, "Title");
    let hex_w = width_of(&h, "Custom color (#RRGGBB)");
    assert!(title_w >= 350.0, "Title input at 960x640: {title_w} px");
    assert!(hex_w >= 100.0, "Color input at 960x640: {hex_w} px");
    assert_all_named(&h, "item view card editor 960x640");
}
