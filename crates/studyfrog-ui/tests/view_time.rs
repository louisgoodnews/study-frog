#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Timing the item page: active time counts, idle time does not, and what is recorded.

mod common;

use common::views::open_model;
use studyfrog_core::uid::Uid;
use studyfrog_core::views::ViewKind;
use studyfrog_service::document::LoadedDoc;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::{EditAction, Msg};
use studyfrog_ui::state::Screen;

const CARD: &str =
    "---\nid: 0000000000000000000000000A\nkind: card\ntitle: Bonjour\n---\nq\n???\na\n";

fn at(m: &mut Model, now: f64) {
    m.state.now = now;
}

fn open(m: &mut Model, text: &str, uid: u128) {
    let doc = studyfrog_md::parse::parse_doc(text).unwrap();
    let loaded = LoadedDoc {
        path: "s/a.md".into(),
        doc,
        hash: 1,
    };
    apply(m, Msg::Navigate(Screen::Editor));
    apply(m, Msg::DocumentLoaded(Uid(uid), Ok(loaded)));
}

/// Move the clock to `to` in one-second steps, with input at every step.
fn work(m: &mut Model, from: u32, to: u32) {
    for t in from..=to {
        at(m, f64::from(t));
        apply(m, Msg::Activity);
        apply(m, Msg::Tick);
    }
}

fn recorded(fx: &[Effect]) -> Vec<(ViewKind, u32)> {
    fx.iter()
        .filter_map(|e| match e {
            Effect::RecordView {
                kind, active_ms, ..
            } => Some((*kind, *active_ms)),
            _ => None,
        })
        .collect()
}

#[test]
fn a_visit_is_recorded_when_the_page_is_left() {
    let mut m = open_model();
    at(&mut m, 100.0);
    open(&mut m, CARD, 0xA);
    assert!(m.state.view.is_some(), "the visit starts with the page");
    work(&mut m, 101, 110);
    let fx = apply(&mut m, Msg::Navigate(Screen::Library));
    assert_eq!(recorded(&fx), [(ViewKind::View, 10_000)]);
    assert!(m.state.view.is_none());
}

#[test]
fn idle_time_is_not_counted_and_the_clock_resumes_with_input() {
    let mut m = open_model();
    at(&mut m, 0.0);
    open(&mut m, CARD, 0xA);
    work(&mut m, 1, 10);
    for t in 11..=200 {
        at(&mut m, f64::from(t));
        apply(&mut m, Msg::Tick);
    }
    work(&mut m, 201, 205);
    let fx = apply(&mut m, Msg::Navigate(Screen::Library));
    let [(ViewKind::View, ms)] = recorded(&fx)[..] else {
        panic!("{fx:?}")
    };
    // 10 s of work, then 60 s of grace after the last input, then 5 s of work after the break.
    assert!((70_000..=76_000).contains(&ms), "{ms}");
}

#[test]
fn a_long_gap_between_ticks_counts_as_a_few_seconds_at_most() {
    let mut m = open_model();
    at(&mut m, 0.0);
    open(&mut m, CARD, 0xA);
    at(&mut m, 3600.0);
    apply(&mut m, Msg::Activity);
    apply(&mut m, Msg::Tick);
    let fx = apply(&mut m, Msg::Navigate(Screen::Library));
    assert_eq!(
        recorded(&fx),
        [(ViewKind::View, 5_000)],
        "a suspended computer is not time on the page"
    );
}

#[test]
fn accidental_opens_are_not_recorded_and_one_second_is_the_least_that_is() {
    let mut short = open_model();
    at(&mut short, 0.0);
    open(&mut short, CARD, 0xA);
    at(&mut short, 0.5);
    apply(&mut short, Msg::Activity);
    apply(&mut short, Msg::Tick);
    assert!(recorded(&apply(&mut short, Msg::Navigate(Screen::Library))).is_empty());

    let mut enough = open_model();
    at(&mut enough, 0.0);
    open(&mut enough, CARD, 0xA);
    work(&mut enough, 1, 1);
    let fx = apply(&mut enough, Msg::Navigate(Screen::Library));
    assert_eq!(recorded(&fx), [(ViewKind::View, 1_000)]);
}

#[test]
fn working_on_the_item_adds_an_edit_event_and_opening_another_item_ends_the_visit() {
    let mut m = open_model();
    at(&mut m, 0.0);
    open(&mut m, CARD, 0xA);
    apply(&mut m, Msg::Edit(EditAction::Title("Salut".into())));
    work(&mut m, 1, 20);
    let other = CARD.replace("0000000000000000000000000A", "0000000000000000000000000B");
    let doc = studyfrog_md::parse::parse_doc(&other).unwrap();
    let loaded = LoadedDoc {
        path: "s/b.md".into(),
        doc,
        hash: 2,
    };
    let fx = apply(&mut m, Msg::DocumentLoaded(Uid(0xB), Ok(loaded)));
    assert_eq!(
        recorded(&fx),
        [(ViewKind::View, 20_000), (ViewKind::Edit, 20_000)]
    );
    assert_eq!(
        m.state.view.as_ref().unwrap().uid,
        Uid(0xB),
        "a new visit has begun"
    );
}

#[test]
fn nothing_is_recorded_when_tracking_is_switched_off() {
    let mut m = open_model();
    apply(&mut m, Msg::SetTrackViews(false));
    assert!(m.state.app.views_untracked);
    at(&mut m, 0.0);
    open(&mut m, CARD, 0xA);
    assert!(m.state.view.is_none(), "no visit is timed");
    work(&mut m, 1, 30);
    assert!(recorded(&apply(&mut m, Msg::Navigate(Screen::Library))).is_empty());
    let fx = apply(&mut m, Msg::SetTrackViews(true));
    assert!(
        matches!(&fx[..], [Effect::SaveAppSettings(a)] if !a.views_untracked),
        "the choice is stored"
    );
}

#[test]
fn switching_off_during_a_visit_drops_it_without_recording() {
    let mut m = open_model();
    at(&mut m, 0.0);
    open(&mut m, CARD, 0xA);
    work(&mut m, 1, 20);
    let fx = apply(&mut m, Msg::SetTrackViews(false));
    assert!(recorded(&fx).is_empty() && m.state.view.is_none());
}
