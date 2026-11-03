#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The red Exit button: leaving at once, or asking first when the editor holds unsaved changes.

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_ui::chrome::ChromeMsg;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::quit_ui::QuitMsg;

fn quit(m: &mut studyfrog_ui::model::Model, q: QuitMsg) {
    apply(m, Msg::Chrome(ChromeMsg::Quit(q)));
}

#[test]
fn exit_leaves_at_once_when_nothing_is_unsaved() {
    let mut m = open_model();
    quit(&mut m, QuitMsg::Ask);
    assert!(m.state.chrome.quit && !m.state.chrome.confirm_quit);
}

#[test]
fn exit_asks_first_with_unsaved_changes_and_keep_working_stays() {
    let (_v, mut m, mut w) = common::flow::opened("quit");
    common::flow::send(&mut m, &mut w, Msg::OpenItem(studyfrog_core::uid::Uid(12)));
    apply(
        &mut m,
        Msg::Edit(studyfrog_ui::msg::EditAction::Title("changed".into())),
    );
    assert!(studyfrog_ui::editor::has_unsaved_changes(&m.state));
    quit(&mut m, QuitMsg::Ask);
    assert!(!m.state.chrome.quit && m.state.chrome.confirm_quit);
    quit(&mut m, QuitMsg::Cancel);
    assert!(!m.state.chrome.quit && !m.state.chrome.confirm_quit);
    quit(&mut m, QuitMsg::Ask);
    quit(&mut m, QuitMsg::Confirm);
    assert!(m.state.chrome.quit);
}

#[test]
fn the_red_button_is_in_the_bar_with_a_name_and_the_button_works() {
    let mut h = harness(open_model());
    h.run();
    assert_all_named(&h, "top bar with exit");
    h.get_by_label("Exit StudyFrog").click();
    h.run();
    assert!(h.state().0.state.chrome.quit);
}
