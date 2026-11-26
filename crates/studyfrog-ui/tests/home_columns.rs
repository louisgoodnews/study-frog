#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Home screen three-column layout bounds (BUG-0057): long item titles must stay inside their own
//! column without overflowing into adjacent columns or past the right edge of the window.

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::{NodeT, Queryable};
use studyfrog_calendar::session::EventKind;
use studyfrog_core::plan::ItemKind;
use studyfrog_core::uid::Uid;
use studyfrog_service::home::{HomeSummary, StackDue, TodaySession, Upcoming};
use studyfrog_store::home::RunRow;
use studyfrog_store::overview::ItemRow;
use studyfrog_ui::home_ui::HomeMsg;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::{Msg, MsgVault};
use studyfrog_ui::state::Screen;

const DAY: i64 = 86_400_000;

const LONG_DUE: &str = "Was passiert mit der Iteminformationsfunktion I_iv, wenn die Differenz zwischen Personenparameter und Itemschwierigkeit zunimmt?";
const LONG_UPCOMING: &str =
    "Klausur Psychologische Diagnostik und probabilistische Testtheorie mit praktischem Teil";
const LONG_RECENT: &str = "Probabilistische Testtheorien (PTT) sind wie ein intelligentes Messinstrument fuer latente Merkmale";

fn long_summary() -> HomeSummary {
    HomeSummary {
        now_ms: 10 * DAY,
        today: studyfrog_core::civil::date_new(2026, 11, 26).unwrap(),
        hour: 10,
        ready_today: 5,
        due_now: 3,
        new_items: 2,
        reviews_today: 4,
        streak_days: 3,
        stacks: vec![StackDue {
            uid: Uid(5),
            folder: "ptt".into(),
            title: "Probabilistische Testtheorien und psychologische Diagnostik".into(),
            due: 17,
            new: 56,
        }],
        attention: Vec::new(),
        due_items: vec![ItemRow {
            uid: Uid(8),
            title: LONG_DUE.into(),
            kind: ItemKind::Card,
            path: "ptt/a.md".into(),
            due: Some(studyfrog_core::time::UnixMs(8 * DAY)),
            status: studyfrog_core::status::Status::New,
        }],
        upcoming: vec![Upcoming {
            title: LONG_UPCOMING.into(),
            kind: EventKind::Exam,
            date: studyfrog_core::civil::date_new(2026, 11, 30).unwrap(),
            start_minute: 600,
        }],
        sessions_today: vec![TodaySession {
            title: "Wiederholung Diagnostik".into(),
            start_minute: 600,
            duration_min: 45,
            stack: Some("Probabilistische Testtheorien".into()),
        }],
        recent_items: vec![ItemRow {
            uid: Uid(7),
            title: LONG_RECENT.into(),
            kind: ItemKind::Note,
            path: "ptt/b.md".into(),
            due: None,
            status: studyfrog_core::status::Status::New,
        }],
        recent_runs: vec![RunRow {
            uid: Uid(9),
            mode: "flip".into(),
            state: "finished".into(),
            started_ms: 7 * DAY,
            rated: 14,
            skipped: 0,
        }],
    }
}

fn model_with_long_titles() -> Model {
    let mut m = open_model();
    apply(&mut m, Msg::Vault(MsgVault::Navigate(Screen::Home)));
    apply(
        &mut m,
        Msg::Home(HomeMsg::Loaded(Ok(Box::new(long_summary())))),
    );
    m
}

fn rect_of(h: &egui_kittest::Harness<'_, common::views::App>, label: &str) -> eframe::egui::Rect {
    let b = h
        .query_all_by_label_contains(label)
        .last()
        .unwrap()
        .accesskit_node()
        .bounding_box()
        .unwrap();
    eframe::egui::Rect::from_min_max(
        eframe::egui::pos2(b.x0 as f32, b.y0 as f32),
        eframe::egui::pos2(b.x1 as f32, b.y1 as f32),
    )
}

fn assert_columns_stay_within_bounds(width: f32, height: f32) {
    let mut h = harness(model_with_long_titles());
    h.set_size(eframe::egui::vec2(width, height));
    h.run_steps(6);

    let coming_hdr = rect_of(&h, "Coming up");
    let recent_hdr = rect_of(&h, "Recent activity");

    let due_item = rect_of(&h, "Iteminformationsfunktion");
    let due_btn = rect_of(&h, "Rehearse Probabilistische");
    let upcoming_item = rect_of(&h, "Klausur Psychologische Diagnostik");
    let recent_item = rect_of(&h, "Probabilistische Testtheorien (PTT)");

    assert!(
        due_item.right() <= coming_hdr.left(),
        "Due now item ({:?}) overflows into Coming up ({:?}) at {width}x{height}",
        due_item,
        coming_hdr
    );
    assert!(
        due_btn.right() <= coming_hdr.left(),
        "Due now stack button ({:?}) overflows into Coming up ({:?}) at {width}x{height}",
        due_btn,
        coming_hdr
    );
    assert!(
        upcoming_item.right() <= recent_hdr.left(),
        "Coming up item ({:?}) overflows into Recent activity ({:?}) at {width}x{height}",
        upcoming_item,
        recent_hdr
    );
    assert!(
        recent_item.right() <= width,
        "Recent activity item ({:?}) overflows window right edge {width} at {width}x{height}",
        recent_item
    );
    assert_all_named(&h, "home columns with long titles");
}

#[test]
fn home_three_columns_do_not_overflow_at_1280x800() {
    assert_columns_stay_within_bounds(1280.0, 800.0);
}

#[test]
fn home_three_columns_do_not_overflow_at_960x640() {
    assert_columns_stay_within_bounds(960.0, 640.0);
}
