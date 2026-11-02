#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The notification center's logic: records, read state, filters, days, targets and what is stored.

mod common;

use common::views::open_model;
use studyfrog_ui::chrome::{ChromeMsg, unread};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::notify::{MAX_AGE_MS, MAX_NOTIFICATIONS, Notification, NotifyEffect, NotifyMsg};
use studyfrog_ui::state::NoticeKind;

const NOW: i64 = 1_800_000_000_000;
const HOUR: i64 = 3_600_000;

fn notice(id: u64, at_ms: i64, kind: NoticeKind) -> Notification {
    Notification {
        id,
        kind,
        text: format!("n{id}"),
        at_ms,
        read: false,
        target: None,
    }
}

/// A model whose vault's stored list has been read (so changes are stored too).
fn loaded() -> Model {
    let mut m = open_model();
    m.state.wall_ms = NOW;
    apply(
        &mut m,
        Msg::Chrome(ChromeMsg::NotificationsLoaded(Ok(Vec::new()))),
    );
    m
}

/// Raise a notification through the status-line notice, the way the app does.
fn raise(m: &mut Model, text: &str) -> Vec<Effect> {
    apply(m, Msg::Chrome(ChromeMsg::SearchEdited("q".into())));
    apply(
        m,
        Msg::Chrome(ChromeMsg::SearchResults("q".into(), Err(text.into()))),
    )
}

fn adds(fx: &[Effect]) -> Vec<&Notification> {
    fx.iter()
        .filter_map(|e| match e {
            Effect::Notifications(NotifyEffect::Add(n)) => Some(n),
            _ => None,
        })
        .collect()
}

fn mine(m: &mut Model, msg: NotifyMsg) -> Vec<Effect> {
    apply(m, Msg::Chrome(ChromeMsg::Notify(msg)))
}

#[test]
fn a_notice_becomes_a_stored_notification_with_an_id_a_time_and_an_unread_flag() {
    let mut m = loaded();
    let fx = raise(&mut m, "disk");
    let added = adds(&fx);
    assert_eq!(added.len(), 1);
    let n = added[0];
    assert_eq!(
        (n.text.as_str(), n.at_ms, n.read, n.kind),
        ("Search failed: disk", NOW, false, NoticeKind::Error)
    );
    raise(&mut m, "again");
    let ids: Vec<u64> = m.state.chrome.notifications.iter().map(|n| n.id).collect();
    assert_eq!(ids.len(), 2);
    assert!(ids[0] < ids[1], "ids rise: {ids:?}");
    assert_eq!(unread(&m.state.chrome), 2);
    let fx = raise(&mut m, "again");
    assert!(
        adds(&fx).is_empty() && m.state.chrome.notifications.len() == 2,
        "an unchanged notice is not added again"
    );
}

#[test]
fn nothing_is_stored_before_the_stored_list_is_read_and_what_came_first_is_merged_then() {
    let mut m = open_model();
    m.state.wall_ms = NOW;
    let fx = raise(&mut m, "early");
    assert!(
        adds(&fx).is_empty(),
        "the list of the vault has not arrived"
    );
    let stored = vec![
        notice(5, NOW - 3 * HOUR, NoticeKind::Info),
        notice(9, NOW - HOUR, NoticeKind::Error),
    ];
    let fx = apply(
        &mut m,
        Msg::Chrome(ChromeMsg::NotificationsLoaded(Ok(stored))),
    );
    let list = &m.state.chrome.notifications;
    assert_eq!(
        list.iter().map(|n| n.text.as_str()).collect::<Vec<_>>(),
        ["n5", "n9", "Search failed: early"]
    );
    let early = list.last().unwrap();
    assert_eq!(early.id, 10, "renumbered after the stored ones");
    assert_eq!(adds(&fx).len(), 1, "and stored now");
    assert!(fx.contains(&Effect::Notifications(NotifyEffect::Prune)));
    let more = raise(&mut m, "later");
    assert_eq!(adds(&more).len(), 1, "from here on everything is stored");
}

#[test]
fn notifications_older_than_thirty_days_are_dropped_when_the_list_is_read() {
    let mut m = open_model();
    m.state.wall_ms = NOW;
    let stored = vec![
        notice(1, NOW - MAX_AGE_MS - 1, NoticeKind::Info),
        notice(2, NOW - MAX_AGE_MS + HOUR, NoticeKind::Info),
    ];
    apply(
        &mut m,
        Msg::Chrome(ChromeMsg::NotificationsLoaded(Ok(stored))),
    );
    assert_eq!(m.state.chrome.notifications.len(), 1);
    assert_eq!(m.state.chrome.notifications[0].id, 2);
}

#[test]
fn reading_and_unreading_one_or_all_are_stored() {
    let mut m = loaded();
    raise(&mut m, "a");
    raise(&mut m, "b");
    let (a, b) = (
        m.state.chrome.notifications[0].id,
        m.state.chrome.notifications[1].id,
    );
    let fx = mine(&mut m, NotifyMsg::MarkRead(a, true));
    assert_eq!(fx, [Effect::Notifications(NotifyEffect::SetRead(a, true))]);
    assert_eq!(unread(&m.state.chrome), 1);
    mine(&mut m, NotifyMsg::MarkRead(a, false));
    assert_eq!(unread(&m.state.chrome), 2);
    let fx = mine(&mut m, NotifyMsg::MarkAll(true));
    assert_eq!(fx, [Effect::Notifications(NotifyEffect::SetAllRead(true))]);
    assert_eq!(unread(&m.state.chrome), 0);
    mine(&mut m, NotifyMsg::MarkAll(false));
    assert_eq!(unread(&m.state.chrome), 2);
    assert!(m.state.chrome.notifications.iter().any(|n| n.id == b));
}

#[test]
fn deleting_one_and_clearing_all_are_stored() {
    let mut m = loaded();
    raise(&mut m, "a");
    raise(&mut m, "b");
    let first = m.state.chrome.notifications[0].id;
    let fx = mine(&mut m, NotifyMsg::Delete(first));
    assert_eq!(fx, [Effect::Notifications(NotifyEffect::Delete(first))]);
    assert_eq!(m.state.chrome.notifications.len(), 1);
    let fx = mine(&mut m, NotifyMsg::Clear);
    assert_eq!(fx, [Effect::Notifications(NotifyEffect::Clear)]);
    assert!(m.state.chrome.notifications.is_empty());
}

#[test]
fn the_list_is_capped_and_the_database_is_told_to_prune() {
    let mut m = loaded();
    let mut pruned = false;
    for i in 0..MAX_NOTIFICATIONS + 3 {
        let fx = raise(&mut m, &format!("m{i}"));
        pruned |= fx.contains(&Effect::Notifications(NotifyEffect::Prune));
    }
    assert_eq!(m.state.chrome.notifications.len(), MAX_NOTIFICATIONS);
    assert!(pruned);
    assert_eq!(m.state.chrome.notifications[0].text, "Search failed: m3");
}
