#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The FrogScript reference open beside the ScriptStudio editor, at the window sizes the app is used
//! at (BUG-0020): the editor keeps a usable width, the reference sits beside it, and its categories,
//! topics and cookbook buttons are laid out inside the window.
use studyfrog_ui::msg::MsgVault;

mod common;

use common::views::{App, open_model};
use eframe::egui::{self, Rect, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use studyfrog_script::cookbook::RECIPES;
use studyfrog_script::wiki::{CATEGORIES, label, topics};
use studyfrog_ui::console_ui::ConsoleMsg;
use studyfrog_ui::model::{Model, apply, frame};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::script_studio::wiki_panel::WikiMsg;
use studyfrog_ui::state::Screen;

/// The sizes the bug was seen at, and the app's smallest comfortable window.
const SIZES: [(f32, f32); 3] = [(960.0, 640.0), (1280.0, 800.0), (1600.0, 900.0)];

/// The editor never gets narrower than this beside the reference.
const EDITOR_MIN_WIDTH: f32 = 300.0;

fn console_with_reference(mut m: Model, w: f32, h: f32) -> Harness<'static, App> {
    apply(&mut m, Msg::Vault(MsgVault::Navigate(Screen::Console)));
    apply(&mut m, Msg::Console(ConsoleMsg::StudioToggleExpanded));
    apply(&mut m, Msg::Console(ConsoleMsg::Wiki(WikiMsg::Toggle)));
    let mut h = Harness::builder()
        .with_size(egui::vec2(w, h))
        .build_ui_state(
            |ui, (model, effects): &mut App| effects.extend(frame(model, ui)),
            (m, Vec::new()),
        );
    h.run_steps(4);
    h
}

/// Whether a rect lies wholly inside the window.
fn inside(r: Rect, size: Vec2) -> bool {
    r.min.x >= 0.0 && r.min.y >= 0.0 && r.max.x <= size.x + 0.5 && r.max.y <= size.y + 0.5
}

/// Whether a rect lies inside the window side to side. A row in a scrolled list may be below the
/// fold, so only the horizontal extent is checked for those.
fn inside_width(r: Rect, size: Vec2) -> bool {
    r.min.x >= 0.0 && r.max.x <= size.x + 0.5
}

#[test]
fn the_editor_keeps_a_usable_width_beside_the_reference_at_each_size() {
    for (w, h) in SIZES {
        let size = Vec2::new(w, h);
        let harness = console_with_reference(open_model(), w, h);
        let editor = harness.get_by_label("Script text").rect();
        let heading = harness.get_by_label("FrogScript reference").rect();
        assert!(
            editor.width() >= EDITOR_MIN_WIDTH,
            "{w}x{h}: the editor is {} px wide",
            editor.width()
        );
        assert!(
            inside(editor, size),
            "{w}x{h}: the editor is outside the window"
        );
        assert!(
            heading.min.x >= editor.max.x - 0.5,
            "{w}x{h}: the reference overlaps the editor"
        );
        assert!(
            inside_width(heading, size),
            "{w}x{h}: the reference runs off the window"
        );
    }
}

#[test]
fn the_reference_lists_every_category_topic_and_cookbook_button_in_the_window() {
    for (w, h) in SIZES {
        let size = Vec2::new(w, h);
        let harness = console_with_reference(open_model(), w, h);
        for category in CATEGORIES {
            let heads = harness.query_all_by_label(label(category)).count();
            assert!(heads >= 1, "{w}x{h}: no heading for {}", label(category));
        }
        for t in topics() {
            let rows: Vec<_> = harness.query_all_by_label(t.name).collect();
            assert!(!rows.is_empty(), "{w}x{h}: no topic {}", t.name);
            assert!(
                rows.iter().any(|r| inside_width(r.rect(), size)),
                "{w}x{h}: the topic {} is outside the window",
                t.name
            );
        }
        let inserts: Vec<_> = harness.query_all_by_label("Insert into editor").collect();
        assert_eq!(
            inserts.len(),
            RECIPES.len(),
            "{w}x{h}: every recipe can be inserted"
        );
        assert!(
            inserts.iter().all(|r| inside_width(r.rect(), size)),
            "{w}x{h}: an Insert into editor button is outside the window"
        );
        let runs: Vec<_> = harness.query_all_by_label("Run in console").collect();
        assert_eq!(runs.len(), RECIPES.iter().filter(|r| r.read_only).count());
        assert!(
            runs.iter().all(|r| inside_width(r.rect(), size)),
            "{w}x{h}: a Run in console button is outside the window"
        );
    }
}

#[test]
fn the_output_pane_stays_in_the_window_with_the_reference_open() {
    for (w, h) in SIZES {
        let size = Vec2::new(w, h);
        let harness = console_with_reference(open_model(), w, h);
        let clear = harness.get_by_label("Clear Output").rect();
        assert!(
            inside(clear, size),
            "{w}x{h}: the output pane is pushed out of the window"
        );
        let editor = harness.get_by_label("Script text").rect();
        assert!(
            clear.min.y > editor.max.y,
            "{w}x{h}: the output pane sits below the editor"
        );
    }
}
