//! The types of the notification center and how they are stored.

use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::{format_ulid, parse_ulid};
use studyfrog_store::notifications::StoredNotification;

use crate::state::{NoticeKind, Screen};

/// Most notifications kept; the oldest go first.
pub const MAX_NOTIFICATIONS: usize = 200;
/// Notifications older than this many milliseconds (30 days) are dropped.
pub const MAX_AGE_MS: i64 = 30 * 86_400_000;

/// What a click on a notification opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotifyTarget {
    /// A screen.
    Screen(Screen),
    /// An item's page.
    Item(Uid),
}

/// A notification.
#[derive(Debug, Clone, PartialEq)]
pub struct Notification {
    /// Identity within the vault.
    pub id: u64,
    /// Information or an error.
    pub kind: NoticeKind,
    /// The message.
    pub text: String,
    /// When it was raised, in Unix milliseconds.
    pub at_ms: i64,
    /// The user has dealt with it.
    pub read: bool,
    /// What a click opens.
    pub target: Option<NotifyTarget>,
}

/// Which notifications the panel lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NotifyFilter {
    /// All.
    #[default]
    All,
    /// Not yet read.
    Unread,
    /// Errors only.
    Errors,
}

/// What the user can do in the panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotifyMsg {
    /// Mark one read or unread.
    MarkRead(u64, bool),
    /// Mark every one read or unread.
    MarkAll(bool),
    /// Delete one.
    Delete(u64),
    /// Delete all.
    Clear,
    /// Choose what is listed.
    Filter(NotifyFilter),
    /// Open the notification's target and mark it read.
    Open(u64),
}

/// What must be stored or asked of the database.
#[derive(Debug, Clone, PartialEq)]
pub enum NotifyEffect {
    /// Read the stored list.
    Load,
    /// Store a notification.
    Add(Notification),
    /// Mark one read or unread.
    SetRead(u64, bool),
    /// Mark all read or unread.
    SetAllRead(bool),
    /// Delete one.
    Delete(u64),
    /// Delete all.
    Clear,
    /// Drop the oldest beyond the caps.
    Prune,
}

const SCREENS: [(Screen, &str); 7] = [
    (Screen::Home, "home"),
    (Screen::Library, "library"),
    (Screen::Calendar, "calendar"),
    (Screen::Board, "board"),
    (Screen::Reports, "reports"),
    (Screen::Settings, "settings"),
    (Screen::Rehearse, "rehearse"),
];

/// The text stored for a target.
pub fn target_text(t: &NotifyTarget) -> Option<String> {
    match t {
        NotifyTarget::Item(uid) => Some(format!("item:{}", format_ulid(*uid))),
        NotifyTarget::Screen(s) => SCREENS
            .iter()
            .find(|(x, _)| x == s)
            .map(|(_, k)| format!("screen:{k}")),
    }
}

fn target_from(text: &str) -> Option<NotifyTarget> {
    if let Some(id) = text.strip_prefix("item:") {
        return parse_ulid(id).map(NotifyTarget::Item);
    }
    let key = text.strip_prefix("screen:")?;
    SCREENS
        .iter()
        .find(|(_, k)| *k == key)
        .map(|(s, _)| NotifyTarget::Screen(*s))
}

/// The record to store.
pub fn to_stored(n: &Notification) -> StoredNotification {
    StoredNotification {
        id: n.id as i64,
        kind: if n.kind == NoticeKind::Error {
            "error"
        } else {
            "info"
        }
        .to_owned(),
        text: n.text.clone(),
        at_ms: n.at_ms,
        read: n.read,
        target: n.target.as_ref().and_then(target_text),
    }
}

/// The notification for a stored record.
pub fn from_stored(s: StoredNotification) -> Notification {
    Notification {
        id: s.id.max(0) as u64,
        kind: if s.kind == "error" {
            NoticeKind::Error
        } else {
            NoticeKind::Info
        },
        text: s.text,
        at_ms: s.at_ms,
        read: s.read,
        target: s.target.as_deref().and_then(target_from),
    }
}
