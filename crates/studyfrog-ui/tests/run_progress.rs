#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The progress bar of a rehearsal run: items done against items still open.

mod common;

use common::runs::act;
use common::views::{harness, running_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::review::Rating;
use studyfrog_core::run::RunMode;
use studyfrog_rehearsal::input::Action;
use studyfrog_ui::run_progress::{Progress, fraction, progress, text};

fn now(m: &studyfrog_ui::model::Model) -> studyfrog_ui::run_progress::Progress {
    progress(&m.state.rehearse.run.as_ref().unwrap().session)
}

#[test]
fn the_text_and_the_fraction_follow_the_items_done_and_open() {
    let p = Progress {
        done: 3,
        open: 5,
        repeats: 0,
    };
    assert_eq!(text(p), "3 done · 5 left");
    assert!((fraction(p) - 0.375).abs() < 1e-6);
    let again = Progress {
        done: 3,
        open: 5,
        repeats: 2,
    };
    assert_eq!(text(again), "3 done · 5 left (2 repeats)");
    assert_eq!(
        text(Progress {
            done: 0,
            open: 2,
            repeats: 1
        }),
        "0 done · 2 left (1 repeat)"
    );
    assert_eq!(
        fraction(Progress {
            done: 0,
            open: 0,
            repeats: 0
        }),
        1.0
    );
}

#[test]
fn an_item_is_open_until_it_is_rated_even_when_its_answer_shows() {
    let mut m = running_model(RunMode::Flip);
    assert_eq!((now(&m).done, now(&m).open), (0, 2));
    act(&mut m, 1.0, Action::Flip);
    assert_eq!(
        (now(&m).done, now(&m).open),
        (0, 2),
        "revealed but not rated"
    );
    act(&mut m, 2.0, Action::Rate(Rating::Good));
    assert_eq!((now(&m).done, now(&m).open), (1, 1));
}

#[test]
fn the_screen_shows_the_bar_with_its_numbers() {
    let mut h = harness(running_model(RunMode::Flip));
    h.run_steps(3);
    assert!(h.query_by_label("Run progress: 0 done · 2 left").is_some());
    act(&mut h.state_mut().0, 1.0, Action::Flip);
    act(&mut h.state_mut().0, 2.0, Action::Rate(Rating::Good));
    h.run_steps(3);
    assert!(h.query_by_label("Run progress: 1 done · 1 left").is_some());
}

#[test]
fn scroll_bars_reserve_their_own_width_so_content_keeps_clear_of_them() {
    let ctx = eframe::egui::Context::default();
    studyfrog_ui::theme::apply(&ctx);
    for theme in [eframe::egui::Theme::Dark, eframe::egui::Theme::Light] {
        let scroll = ctx.style_of(theme).spacing.scroll;
        assert!(scroll.floating_allocated_width >= 10.0, "{theme:?}");
        assert!(scroll.bar_outer_margin >= 3.0, "{theme:?}");
    }
}
