#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The Leeches tab and its actions (STORY-0104).

mod common;

use common::analytics::analytics;
use common::reports::bundle;
use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::status::Status;
use studyfrog_core::uid::Uid;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::explorer_ui::{ExplorerMsg, ReportTab};
use studyfrog_ui::library_bulk::LibraryEffect;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

fn on_leeches() -> studyfrog_ui::model::Model {
    let mut m = open_model();
    m.state.screen = Screen::Reports;
    m.state.explorer.tab = ReportTab::Leeches;
    let mut b = bundle();
    b.analytics = Box::new(analytics());
    m.state.reports = Some(b);
    m
}

#[test]
fn leeches_and_pairs_are_listed_with_their_reasons_and_numbers() {
    let mut h = harness(on_leeches());
    h.run_steps(4);
    for text in [
        "Subjonctif",
        "- forgotten again and again; takes far longer than the stack's other items",
        "5 lapse(s), 9 review(s), 2 hard or forgotten in a row, 4.0 times the stack's usual time, 0.5 days of stability per review",
        "Ionic bond",
        "Covalent bond",
        "forgotten within a day of each other 3 time(s), 40 % shared words, 1 shared tag(s)",
        "Drill Ionic bond and Covalent bond together",
    ] {
        assert!(h.query_by_label(text).is_some(), "missing {text}");
    }
    assert_all_named(&h, "leeches report");
}

#[test]
fn the_actions_open_pause_and_drill() {
    let mut h = harness(on_leeches());
    h.run_steps(4);
    h.get_by_label("Pause Subjonctif").click();
    h.run_steps(2);
    let mut m = on_leeches();
    let fx = apply(&mut m, Msg::Explorer(ExplorerMsg::Pause(Uid(7))));
    assert_eq!(
        fx,
        [Effect::Library(LibraryEffect::SetStatus(
            Uid(7),
            Some(Status::Paused)
        ))]
    );
    let fx = apply(
        &mut m,
        Msg::Explorer(ExplorerMsg::Drill(vec![Uid(10), Uid(11)])),
    );
    assert_eq!(m.state.screen, Screen::Rehearse);
    assert!(m.state.rehearse.starting);
    assert!(fx.contains(&Effect::Library(LibraryEffect::Drill(vec![
        Uid(10),
        Uid(11)
    ]))));
    let again = apply(&mut m, Msg::Explorer(ExplorerMsg::Drill(vec![Uid(10)])));
    assert!(again.is_empty(), "one run at a time");
    assert!(m.state.notice.is_some());
    let mut m = on_leeches();
    let fx = apply(&mut m, Msg::OpenItem(Uid(7)));
    assert!(!fx.is_empty(), "opening the item loads it");
}

#[test]
fn clicking_drill_on_a_pair_sends_both_items() {
    let mut h = harness(on_leeches());
    h.run_steps(4);
    h.get_by_label("Drill Ionic bond and Covalent bond together")
        .click();
    h.run_steps(2);
    let s = &h.state().0.state;
    assert_eq!(s.screen, Screen::Rehearse);
    assert!(s.rehearse.starting);
}

#[test]
fn a_stacks_reports_tab_lists_only_its_own_leeches_and_pairs() {
    let mut m = open_model();
    m.state.app.stack_tabs = vec!["french:reports".into()];
    apply(
        &mut m,
        Msg::Select(studyfrog_ui::state::Selection::Stack("french".into())),
    );
    let mut b = bundle();
    b.analytics = Box::new(analytics());
    m.state.reports = Some(b);
    let mut h = harness(m);
    h.run_steps(4);
    assert!(h.query_by_label("Leeches").is_some());
    assert!(
        h.query_by_label("Pause Subjonctif").is_some(),
        "French's leech"
    );
    assert!(
        h.query_by_label("No pairs that get confused.").is_some(),
        "the chemistry pair belongs to another stack"
    );
}
