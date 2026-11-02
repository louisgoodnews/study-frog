#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The notification center's lists: filters, days, what a click opens and what is stored.

mod common;

use common::views::open_model;
use studyfrog_core::uid::Uid;
use studyfrog_ui::chrome::{ChromeMsg, Menu};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::notify::{
    Notification, NotifyEffect, NotifyFilter, NotifyMsg, NotifyTarget, by_day, from_stored,
    to_stored, visible,
};
use studyfrog_ui::state::{NoticeKind, Screen};

const NOW: i64 = 1_800_000_000_000;
const HOUR: i64 = 3_600_000;
const DAY: i64 = 86_400_000;

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

fn loaded() -> Model {
    let mut m = open_model();
    m.state.wall_ms = NOW;
    apply(
        &mut m,
        Msg::Chrome(ChromeMsg::NotificationsLoaded(Ok(Vec::new()))),
    );
    m
}

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
fn filters_list_unread_or_errors_newest_first() {
    let mut read = notice(2, NOW - 2 * HOUR, NoticeKind::Info);
    read.read = true;
    let list = vec![
        notice(1, NOW - 3 * HOUR, NoticeKind::Error),
        read,
        notice(3, NOW - HOUR, NoticeKind::Info),
    ];
    let ids = |f| visible(&list, f).iter().map(|n| n.id).collect::<Vec<_>>();
    assert_eq!(ids(NotifyFilter::All), [3, 2, 1]);
    assert_eq!(ids(NotifyFilter::Unread), [3, 1]);
    assert_eq!(ids(NotifyFilter::Errors), [1]);
    let mut m = loaded();
    assert!(mine(&mut m, NotifyMsg::Filter(NotifyFilter::Errors)).is_empty());
    assert_eq!(m.state.chrome.notify_filter, NotifyFilter::Errors);
}

#[test]
fn notifications_are_grouped_under_today_yesterday_and_their_date_in_local_time() {
    let list = vec![
        notice(1, NOW - 40 * DAY, NoticeKind::Info),
        notice(2, NOW - DAY, NoticeKind::Info),
        notice(3, NOW - 1000, NoticeKind::Info),
    ];
    let shown = visible(&list, NotifyFilter::All);
    let groups = by_day(&shown, NOW, 0);
    let labels: Vec<&str> = groups.iter().map(|(l, _)| l.as_str()).collect();
    assert_eq!(&labels[..2], ["Today", "Yesterday"]);
    assert!(labels[2].starts_with("20"), "{labels:?}");
    assert_eq!(groups.iter().map(|(_, l)| l.len()).sum::<usize>(), 3);
    let same_day = vec![
        notice(1, NOW - 1000, NoticeKind::Info),
        notice(2, NOW - 2000, NoticeKind::Info),
    ];
    assert_eq!(
        by_day(&visible(&same_day, NotifyFilter::All), NOW, 0).len(),
        1
    );
}

#[test]
fn opening_a_notification_with_a_target_goes_there_marks_it_read_and_closes_the_panel() {
    let mut m = loaded();
    raise(&mut m, "x");
    let id = m.state.chrome.notifications[0].id;
    m.state.chrome.notifications[0].target = Some(NotifyTarget::Screen(Screen::Calendar));
    apply(
        &mut m,
        Msg::Chrome(ChromeMsg::ToggleMenu(Menu::Notifications)),
    );
    let fx = mine(&mut m, NotifyMsg::Open(id));
    assert_eq!(m.state.screen, Screen::Calendar);
    assert_eq!(m.state.chrome.menu, None);
    assert!(m.state.chrome.notifications[0].read);
    assert!(
        fx.contains(&Effect::LoadSessions)
            && fx.contains(&Effect::Notifications(NotifyEffect::SetRead(id, true)))
    );

    raise(&mut m, "y");
    let second = m.state.chrome.notifications[1].id;
    m.state.chrome.notifications[1].target = Some(NotifyTarget::Item(Uid(0xA)));
    let fx = mine(&mut m, NotifyMsg::Open(second));
    assert!(fx.contains(&Effect::LoadDocument(Uid(0xA))), "{fx:?}");
}

#[test]
fn stored_records_read_back_with_their_targets_and_unknown_ones_are_ignored() {
    let mut n = notice(7, NOW, NoticeKind::Error);
    for target in [
        Some(NotifyTarget::Screen(Screen::Calendar)),
        Some(NotifyTarget::Item(Uid(0xBEEF))),
        None,
    ] {
        n.target = target;
        assert_eq!(from_stored(to_stored(&n)), n);
    }
    let mut stored = to_stored(&n);
    stored.target = Some("screen:nowhere".into());
    assert_eq!(from_stored(stored.clone()).target, None);
    stored.target = Some("item:not-an-id".into());
    assert_eq!(from_stored(stored).target, None);
}

#[test]
fn a_notification_list_belongs_to_its_vault() {
    let mut m = loaded();
    raise(&mut m, "mine");
    let fx = apply(&mut m, Msg::CloseVault);
    assert!(
        m.state.chrome.notifications.is_empty(),
        "leaving the vault leaves its list"
    );
    assert!(adds(&fx).is_empty());
    let fx = apply(&mut m, Msg::VaultOpened(Ok(common::views::info())));
    assert!(
        fx.contains(&Effect::Notifications(NotifyEffect::Load)),
        "the next vault's list is asked for"
    );
}
