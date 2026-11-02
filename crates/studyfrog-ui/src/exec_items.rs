//! Performing the item-list effects of the stack browser. Thin: the queries live in the store.

use studyfrog_store::overview::{list_items, status_counts};

use crate::exec::Worker;
use crate::msg::Msg;
use crate::state::Selection;

/// The items directly in the selected stack (or the inbox).
pub fn items(w: &Worker, selection: &Selection) -> Msg {
    let folder = match selection {
        Selection::Stack(f) => Some(f.as_str()),
        Selection::Inbox | Selection::Nothing => None,
    };
    match &w.session {
        Some(s) => Msg::ItemsLoaded(list_items(&s.conn, folder).map_err(|e| e.to_string())),
        None => Msg::ItemsLoaded(Ok(Vec::new())),
    }
}

/// How many items of the selected stack and its sub-stacks are in each status.
pub fn counts(w: &Worker, selection: &Selection) -> Msg {
    let (Selection::Stack(folder), Some(s)) = (selection, &w.session) else {
        return Msg::StatusCountsLoaded(Ok(Vec::new()));
    };
    Msg::StatusCountsLoaded(status_counts(&s.conn, folder).map_err(|e| e.to_string()))
}

/// Store a recorded view. A failure is not shown: losing a few seconds of timing must not interrupt the user.
pub fn record_view(
    w: &Worker,
    (item, kind, active_ms): (
        studyfrog_core::uid::Uid,
        studyfrog_core::views::ViewKind,
        u32,
    ),
    now_ns: i64,
) -> Vec<Msg> {
    if let Some(s) = &w.session {
        let at = studyfrog_core::time::UnixMs(now_ns / 1_000_000);
        let _ = studyfrog_store::views::insert_view(
            &s.conn,
            &studyfrog_core::views::ViewEvent {
                item,
                kind,
                at,
                active_ms,
            },
        );
    }
    Vec::new()
}

/// Store a change to the notifications, or read the list. A failed write is not reported: the list on screen is right.
pub fn notifications(w: &Worker, op: crate::notify::NotifyEffect, now_ns: i64) -> Vec<Msg> {
    use crate::notify::{MAX_AGE_MS, MAX_NOTIFICATIONS, NotifyEffect, from_stored, to_stored};
    use studyfrog_store::notifications as db;
    let Some(s) = &w.session else {
        return Vec::new();
    };
    let c = &s.conn;
    let now_ms = now_ns / 1_000_000;
    let _ = match op {
        NotifyEffect::Load => {
            let list = db::load_notifications(c).map(|l| l.into_iter().map(from_stored).collect());
            let msg =
                crate::chrome::ChromeMsg::NotificationsLoaded(list.map_err(|e| e.to_string()));
            return vec![Msg::Chrome(msg)];
        }
        NotifyEffect::Add(n) => db::insert_notification(c, &to_stored(&n)),
        NotifyEffect::SetRead(id, read) => db::set_notification_read(c, id as i64, read),
        NotifyEffect::SetAllRead(read) => db::set_all_notifications_read(c, read),
        NotifyEffect::Delete(id) => db::delete_notification(c, id as i64),
        NotifyEffect::Clear => db::clear_notifications(c),
        NotifyEffect::Prune => db::prune_notifications(c, MAX_NOTIFICATIONS, now_ms - MAX_AGE_MS),
    };
    Vec::new()
}
