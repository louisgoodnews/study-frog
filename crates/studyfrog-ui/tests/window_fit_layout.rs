#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The header, the rehearsal top strip, the Reports tab row, the week view and the toasts stay inside the
//! window and off each other at the sizes the app is used at (BUG-0021, BUG-0032, BUG-0035, BUG-0036).

mod common;

use common::views::{App, open_model, running_model};
use eframe::egui::{self, Rect};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use studyfrog_core::civil::date_new;
use studyfrog_core::run::RunMode;
use studyfrog_ui::calendar_mode::CalendarViewMode;
use studyfrog_ui::modal_types::{Toast, ToastAction};
use studyfrog_ui::model::{Model, frame};
use studyfrog_ui::state::Screen;

fn build(m: Model, w: f32, h: f32) -> Harness<'static, App> {
    let mut h = Harness::builder()
        .with_size(egui::vec2(w, h))
        .build_ui_state(
            |ui, (model, effects): &mut App| effects.extend(frame(model, ui)),
            (m, Vec::new()),
        );
    h.run_steps(4);
    h
}

/// Whether a rect lies inside the window from side to side.
fn inside_width(r: Rect, w: f32) -> bool {
    r.min.x >= 0.0 && r.max.x <= w + 0.5
}

#[test]
fn the_header_brand_search_and_create_do_not_overlap_at_the_minimum_size() {
    let (w, h) = (640.0, 420.0);
    let harness = build(open_model(), w, h);
    // At this width the brand text is left out, so the sidebar toggle is the left-hand item.
    let toggle = harness.get_by_label("Show or hide the sidebar").rect();
    let search = harness.get_by_label("Search items").rect();
    let create = harness.get_by_label("+ Create").rect();
    assert!(
        !toggle.intersects(search),
        "the sidebar toggle {toggle:?} is drawn over the search {search:?}"
    );
    assert!(
        !search.intersects(create),
        "the search {search:?} is drawn over Create {create:?}"
    );
    let bell = harness.get_by_label("Notifications").rect();
    assert!(
        !create.intersects(bell),
        "Create {create:?} is drawn over the notifications button {bell:?}"
    );
    assert!(inside_width(search, w), "the search runs off the window");
}

#[test]
fn the_rehearsal_top_strip_buttons_do_not_overlap_at_the_minimum_size() {
    let (w, h) = (640.0, 420.0);
    let harness = build(running_model(RunMode::Answer), w, h);
    let rects: Vec<(&str, Rect)> = [
        "Type answers",
        "Comments",
        "Pause (P)",
        "Edit (Ctrl+E)",
        "Stop (Esc)",
    ]
    .into_iter()
    .map(|name| (name, harness.get_by_label(name).rect()))
    .collect();
    for (name, r) in &rects {
        assert!(inside_width(*r, w), "{name} runs past the window: {r:?}");
    }
    for (i, (a, ra)) in rects.iter().enumerate() {
        for (b, rb) in &rects[i + 1..] {
            assert!(!ra.intersects(*rb), "{a} overlaps {b}");
        }
    }
    let hint = harness.get_by_label_contains("Esc stop").rect();
    assert!(inside_width(hint, w), "the hint line runs past the window");
}

#[test]
fn every_reports_tab_is_inside_the_window_at_960_by_640() {
    let (w, h) = (960.0, 640.0);
    let mut m = open_model();
    m.state.screen = Screen::Reports;
    let harness = build(m, w, h);
    for label in [
        "Performance",
        "Memory and forecast",
        "Familiarity",
        "Self-assessment",
        "Cognitive profile",
        "Runs",
        "Habits",
        "Leeches",
        "Workload",
    ] {
        let r = harness.get_by_label(label).rect();
        assert!(
            inside_width(r, w),
            "the {label} tab runs past the window: {r:?}"
        );
    }
}

#[test]
fn the_whole_week_fits_the_window_at_1280_and_1600() {
    for (w, h) in [(1280.0, 800.0), (1600.0, 900.0)] {
        let mut m = open_model();
        m.state.screen = Screen::Calendar;
        m.state.planner.loaded = true;
        m.state.planner.view_mode = CalendarViewMode::Week;
        m.state.planner.selected = date_new(2026, 11, 23).unwrap();
        let harness = build(m, w, h);
        for day in [
            "Mon 23", "Tue 24", "Wed 25", "Thu 26", "Fri 27", "Sat 28", "Sun 29",
        ] {
            let r = harness.get_by_label(day).rect();
            assert!(
                inside_width(r, w),
                "{w}x{h}: {day} runs past the window: {r:?}"
            );
        }
    }
}

#[test]
fn reminder_toasts_do_not_cover_the_day_views_add_session_button() {
    let mut m = open_model();
    m.state.screen = Screen::Calendar;
    m.state.planner.loaded = true;
    m.state.planner.view_mode = CalendarViewMode::Day;
    m.state.planner.selected = date_new(2026, 11, 23).unwrap();
    for title in ["Essay 2", "Kurzquiz"] {
        m.state.toasts.push(Toast {
            title: "Homework due".into(),
            detail: format!("{title} (due in 1 h 8 min, 2026-11-23 22:30)"),
            action: ToastAction::OpenCalendar,
            expires_ms: None,
        });
    }
    let (w, h) = (1280.0, 800.0);
    let harness = build(m, w, h);
    let add = harness.get_by_label("+ Add session").rect();
    let toasts = harness.query_all_by_label_contains("(due in ").count();
    assert_eq!(toasts, 2, "both reminder toasts are shown");
    for toast in harness.query_all_by_label_contains("(due in ") {
        let r = toast.rect();
        assert!(
            !r.intersects(add),
            "a toast {r:?} covers + Add session {add:?}"
        );
        assert!(
            r.max.y <= h && r.min.x >= 0.0,
            "a toast runs off the window"
        );
    }
}
