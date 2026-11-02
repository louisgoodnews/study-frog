#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The notification panel under the bell, headless.

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_ui::chrome::{ChromeMsg, Menu, unread};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::notify::{Notification, NotifyFilter, NotifyTarget};
use studyfrog_ui::state::{NoticeKind, Screen};

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

fn n(id: u64, text: &str, kind: NoticeKind, read: bool, age_ms: i64) -> Notification {
    Notification {
        id,
        kind,
        text: text.into(),
        at_ms: now_ms() - age_ms,
        read,
        target: None,
    }
}

fn model(list: Vec<Notification>) -> Model {
    let mut m = open_model();
    m.state.wall_ms = now_ms();
    apply(
        &mut m,
        Msg::Chrome(ChromeMsg::NotificationsLoaded(Ok(list))),
    );
    apply(
        &mut m,
        Msg::Chrome(ChromeMsg::ToggleMenu(Menu::Notifications)),
    );
    m
}

fn three() -> Vec<Notification> {
    vec![
        n(1, "Saved.", NoticeKind::Info, true, 5_000),
        n(
            2,
            "Could not save the subjects: disk full",
            NoticeKind::Error,
            false,
            4_000,
        ),
        n(3, "Created \"Einführung\".", NoticeKind::Info, false, 3_000),
    ]
}

#[test]
fn the_panel_lists_everything_under_today_with_the_unread_count() {
    let mut h = harness(model(three()));
    h.run_steps(2);
    assert!(h.query_by_label("Notifications (2 unread)").is_some());
    assert!(h.query_by_label("Today").is_some());
    for text in [
        "Saved.",
        "Could not save the subjects: disk full",
        "Created \"Einführung\".",
    ] {
        assert!(h.query_by_label(text).is_some(), "{text}");
    }
    assert_all_named(&h, "notification panel");
}

#[test]
fn mark_read_and_mark_unread_and_delete_act_on_one_notification() {
    let mut h = harness(model(three()));
    h.run_steps(2);
    h.get_by_label("Mark read: Created \"Einführung\".").click();
    h.run_steps(2);
    assert_eq!(unread(&h.state().0.state.chrome), 1);
    h.get_by_label("Mark unread: Saved.").click();
    h.run_steps(2);
    assert_eq!(unread(&h.state().0.state.chrome), 2);
    h.get_by_label("Delete: Saved.").click();
    h.run_steps(2);
    assert_eq!(h.state().0.state.chrome.notifications.len(), 2);
    assert!(h.query_by_label("Saved.").is_none());
}

#[test]
fn mark_all_read_and_clear_all_work_from_the_header() {
    let mut h = harness(model(three()));
    h.run_steps(2);
    h.get_by_label("Mark all read").click();
    h.run_steps(2);
    assert_eq!(unread(&h.state().0.state.chrome), 0);
    assert!(h.query_by_label("Notifications (0 unread)").is_some());
    h.get_by_label("Clear all").click();
    h.run_steps(2);
    assert!(h.state().0.state.chrome.notifications.is_empty());
    assert!(h.query_by_label("Nothing yet.").is_some());
}

#[test]
fn the_filters_show_unread_or_errors_and_say_when_nothing_matches() {
    let mut h = harness(model(three()));
    h.run_steps(2);
    h.get_by_label("Show: Errors").click();
    h.run_steps(2);
    assert!(
        h.query_by_label("Could not save the subjects: disk full")
            .is_some()
    );
    assert!(h.query_by_label("Saved.").is_none());
    h.get_by_label("Show: Unread").click();
    h.run_steps(2);
    assert!(
        h.query_by_label("Saved.").is_none()
            && h.query_by_label("Created \"Einführung\".").is_some()
    );
    assert_eq!(h.state().0.state.chrome.notify_filter, NotifyFilter::Unread);

    let mut quiet = harness(model(vec![n(1, "Saved.", NoticeKind::Info, true, 1_000)]));
    quiet.run_steps(2);
    quiet.get_by_label("Show: Errors").click();
    quiet.run_steps(2);
    assert!(quiet.query_by_label("No errors.").is_some());
    quiet.get_by_label("Show: Unread").click();
    quiet.run_steps(2);
    assert!(quiet.query_by_label("Everything is read.").is_some());
}

#[test]
fn a_notification_with_a_target_is_a_link_that_opens_it() {
    let mut list = three();
    list[1].target = Some(NotifyTarget::Screen(Screen::Settings));
    let mut h = harness(model(list));
    h.run_steps(2);
    h.get_by_label("Could not save the subjects: disk full")
        .click();
    h.run_steps(2);
    let state = &h.state().0.state;
    assert_eq!(state.screen, Screen::Settings);
    assert_eq!(state.chrome.menu, None, "the panel closes");
    assert!(
        state
            .chrome
            .notifications
            .iter()
            .find(|n| n.id == 2)
            .unwrap()
            .read
    );
}
