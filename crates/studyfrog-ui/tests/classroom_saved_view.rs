#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The saved-session browser's rows and layout (TASK-0295): the title, topic, date and counts, the empty
//! state, and the phone and desktop widths. Headless; the worker reads the sample vault and no model runs.
use classroom_common::{online, sized, started};
use common::flow::send;
use egui_kittest::kittest::Queryable;
use studyfrog_core::civil::{date_from_days, format_date};
use studyfrog_ui::classroom_types::ClassroomMsg;
use studyfrog_ui::model::Model;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

mod classroom_common;
mod common;

fn leave(m: &mut Model, w: &mut studyfrog_ui::exec::Worker) {
    send(m, w, Msg::Classroom(ClassroomMsg::Leave));
}

fn advance(m: &mut Model, w: &mut studyfrog_ui::exec::Worker) {
    send(m, w, Msg::Classroom(ClassroomMsg::Advance));
}

fn saved_one() -> (
    common::TempDir,
    Model,
    studyfrog_ui::exec::Worker,
    common::TempDir,
) {
    let (vault, mut m, mut w) = started();
    let cfg = online(&mut w);
    advance(&mut m, &mut w);
    leave(&mut m, &mut w);
    (vault, m, w, cfg)
}

#[test]
fn the_browser_lists_each_saved_session_with_its_title_topic_date_and_counts() {
    let (_vault, m, _w, _cfg) = saved_one();
    let row = m.state.classroom.saved.first().unwrap().clone();
    assert_eq!(row.title, "Classroom");
    assert_eq!(row.turns, 1);
    assert_eq!(row.tasks, 0);
    let stack = m.state.vault.as_ref().unwrap().overview.stacks[0]
        .title
        .clone();
    let date = format_date(date_from_days(row.created_at.0.div_euclid(86_400_000)));
    let meta = format!("{date} · 1 turns · 0 tasks");

    let mut m = m;
    m.state.screen = Screen::Classroom;
    let h = sized(m, 960.0, 640.0);
    assert!(
        h.query_by_label("Resume Classroom").is_some(),
        "the row is drawn"
    );
    assert!(
        h.query_by_label(&format!("Topic: {stack}")).is_some(),
        "the topic names the stack"
    );
    assert!(h.query_by_label(&meta).is_some(), "the date and counts");
}

#[test]
fn an_empty_browser_says_so_and_shows_no_rows() {
    let (_vault, mut m, _w) = common::flow::opened("classroom-browser-empty");
    assert!(m.state.classroom.saved.is_empty());
    m.state.screen = Screen::Classroom;
    let h = sized(m, 960.0, 640.0);
    assert!(h.query_by_label_contains("No saved sessions yet").is_some());
    assert!(h.query_by_label("Resume Classroom").is_none());
}

#[test]
fn the_browser_fits_phone_and_desktop_widths_without_horizontal_overflow() {
    for (w_px, h_px) in [(360.0, 640.0), (960.0, 640.0), (1280.0, 800.0)] {
        let (_vault, m, _w, _cfg) = saved_one();
        let mut m = m;
        m.state.screen = Screen::Classroom;
        // A phone has no room for the 150-point screen list beside the form, so it is folded away.
        m.state.chrome.sidebar_hidden = w_px < 720.0;
        let h = sized(m, w_px, h_px);
        for label in [
            "Resume Classroom",
            "Export Classroom to Markdown",
            "Delete Classroom",
        ] {
            let r = h.get_by_label(label).rect();
            assert!(
                r.min.x >= 0.0 && r.max.x <= w_px,
                "{label} {r:?} fits the {w_px}-pixel width"
            );
        }
    }
}
