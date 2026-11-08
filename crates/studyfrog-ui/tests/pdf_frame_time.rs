#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! STORY-0085: the PDF screen with a 500-page document, measured frame by frame without a window.
//! Each frame is the whole CPU side of a real frame (the update, the layout of the screen and the
//! tessellation into triangles); only the upload to the graphics card is missing.

mod common;
mod pdf_frame_common;

use pdf_frame_common::{
    OVER_PAGE, budget_ms, frame_of, open_big, renders, run, send, settle, thumb_renders,
    timed_frame, view, wheel_at, wheel_over_side,
};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::pdf_types::{PdfMsg, Side, Zoom};

#[test]
fn the_thumbnail_list_asks_only_for_the_rows_in_view_and_scrolling_asks_for_the_new_ones() {
    let mut f = open_big("thumbs-500");
    send(&mut f, PdfMsg::ShowSide(Side::Pages));
    let first = frame_of(&mut f, Vec::new());
    let asked = thumb_renders(&first);
    assert!(
        (1..=8).contains(&asked.len()),
        "500 pages, but only the rows in view are asked for: {asked:?}"
    );
    settle(&mut f, first);
    let mut total = asked.len();
    let mut most_in_one_frame = 0;
    for _ in 0..120 {
        let fx = frame_of(&mut f, wheel_over_side(-60.0));
        let n = thumb_renders(&fx).len();
        most_in_one_frame = most_in_one_frame.max(n);
        total += n;
        run(&mut f, fx);
    }
    let pages = view(&f).pictures.iter().map(|(k, _)| k.page).max().unwrap();
    eprintln!(
        "thumbnails: {} asked at first, {total} after scrolling to page {pages}, at most {most_in_one_frame} in one frame",
        asked.len()
    );
    assert!(pages >= 30, "the list scrolled well down: page {pages}");
    assert!(
        total <= pages + 10,
        "only rows passed by are asked for ({total} for {pages} rows)"
    );
    assert!(most_in_one_frame <= 8);
}

#[test]
fn a_zoom_change_asks_the_worker_and_the_frame_keeps_showing_the_old_picture() {
    let mut f = open_big("zoom-500");
    let first = frame_of(&mut f, Vec::new());
    settle(&mut f, first);
    let before = view(&f).pictures.len();
    studyfrog_ui::model::apply(&mut f.model, Msg::Pdf(PdfMsg::SetZoom(Zoom::Percent(300))));
    let (fx, ms) = timed_frame(&mut f, Vec::new());
    let asked = renders(&fx);
    assert_eq!(asked.len(), 1, "the sharp page is asked of the worker");
    assert!(asked[0].scale_milli >= 3000);
    assert_eq!(
        view(&f).pictures.len(),
        before,
        "nothing is drawn in the frame itself"
    );
    assert!(
        ms < budget_ms(),
        "the frame after a zoom change took {ms:.1} ms"
    );
    run(&mut f, fx);
    assert!(view(&f).pictures.iter().any(|(k, _)| k.scale_milli >= 3000));
}

#[test]
fn scrolling_the_thumbnails_and_reading_stay_inside_the_frame_budget() {
    let mut f = open_big("budget-500");
    send(&mut f, PdfMsg::ShowSide(Side::Pages));
    let first = frame_of(&mut f, Vec::new());
    settle(&mut f, first);
    send(&mut f, PdfMsg::SetZoom(Zoom::Percent(200)));
    let first = frame_of(&mut f, Vec::new());
    settle(&mut f, first);
    let mut times = Vec::new();
    for i in 0..90 {
        // Thumbnails, then panning the page down and sideways, then still frames.
        let wheel = match i {
            0..40 => wheel_over_side(-40.0),
            40..60 => wheel_at(OVER_PAGE, eframe::egui::vec2(0.0, -40.0)),
            60..75 => wheel_at(OVER_PAGE, eframe::egui::vec2(-30.0, 0.0)),
            _ => Vec::new(),
        };
        let (fx, ms) = timed_frame(&mut f, wheel);
        run(&mut f, fx);
        times.push(ms);
    }
    times.sort_by(f64::total_cmp);
    let (median, p95, worst) = (times[45], times[85], times[89]);
    eprintln!(
        "frame (update + layout + tessellation), 500 pages at 200 %, scrolling thumbnails and panning: median {median:.2} ms, p95 {p95:.2} ms, worst {worst:.2} ms; budget at 60 fps 16.7 ms"
    );
    assert!(median < budget_ms(), "median frame {median:.1} ms");
}

#[test]
fn frames_on_a_steady_screen_ask_for_nothing_and_keep_the_cache_order() {
    let mut f = open_big("steady-500");
    send(&mut f, PdfMsg::ShowSide(Side::Pages));
    let first = frame_of(&mut f, Vec::new());
    settle(&mut f, first);
    let order: Vec<_> = view(&f).pictures.iter().map(|(k, _)| *k).collect();
    for _ in 0..5 {
        let fx = frame_of(&mut f, Vec::new());
        assert!(fx.is_empty(), "a steady screen asks for nothing: {fx:?}");
    }
    let now: Vec<_> = view(&f).pictures.iter().map(|(k, _)| *k).collect();
    assert_eq!(order, now);
}
