//! The notification center: the bell's list as records the user can read, delete and filter, kept in the vault's
//! database so they survive a restart. Pure: the clock is `state.wall_ms`, storage is an effect.

use studyfrog_core::civil::{Date, days_between, format_date, utc_to_local};
use studyfrog_core::time::UnixMs;

use crate::chrome::Menu;
use crate::effect::Effect;
use crate::msg::Msg;
use crate::state::{Notice, NoticeKind, State};

pub use crate::notify_types::{
    MAX_AGE_MS, MAX_NOTIFICATIONS, Notification, NotifyEffect, NotifyFilter, NotifyMsg,
    NotifyTarget, from_stored, target_text, to_stored,
};

/// How many are not yet read.
pub fn unread_count(list: &[Notification]) -> usize {
    list.iter().filter(|n| !n.read).count()
}

/// The notifications the filter lists, newest first.
pub fn visible(list: &[Notification], filter: NotifyFilter) -> Vec<&Notification> {
    let mut shown: Vec<&Notification> = list
        .iter()
        .filter(|n| match filter {
            NotifyFilter::All => true,
            NotifyFilter::Unread => !n.read,
            NotifyFilter::Errors => n.kind == NoticeKind::Error,
        })
        .collect();
    shown.sort_by(|a, b| b.at_ms.cmp(&a.at_ms).then(b.id.cmp(&a.id)));
    shown
}

fn local_date(at_ms: i64, offset_minutes: i32) -> Date {
    utc_to_local(UnixMs(at_ms), offset_minutes).date
}

/// The notifications under "Today", "Yesterday" or their date, newest group first; `shown` is newest first.
pub fn by_day<'a>(
    shown: &[&'a Notification],
    now_ms: i64,
    offset_minutes: i32,
) -> Vec<(String, Vec<&'a Notification>)> {
    let today = local_date(now_ms, offset_minutes);
    let mut groups: Vec<(Date, Vec<&Notification>)> = Vec::new();
    for n in shown {
        let day = local_date(n.at_ms, offset_minutes);
        match groups.last_mut().filter(|(d, _)| *d == day) {
            Some((_, list)) => list.push(n),
            None => groups.push((day, vec![n])),
        }
    }
    groups
        .into_iter()
        .map(|(d, list)| {
            let label = match days_between(d, today) {
                0 => "Today".to_owned(),
                1 => "Yesterday".to_owned(),
                _ => format_date(d),
            };
            (label, list)
        })
        .collect()
}

fn persisting(state: &State) -> bool {
    state.chrome.notify_vault.is_some() && state.chrome.notify_loaded
}

/// Raise a notification. It is stored when the vault's list has been read.
pub fn push(
    state: State,
    (kind, text): (NoticeKind, String),
    target: Option<NotifyTarget>,
) -> (State, Vec<Effect>) {
    let mut state = state;
    let saving = persisting(&state);
    let id = state.chrome.next_id.max(1);
    let n = Notification {
        id,
        kind,
        text,
        at_ms: state.wall_ms,
        read: false,
        target,
    };
    state.chrome.next_id = id + 1;
    let mut effects = Vec::new();
    if saving {
        effects.push(Effect::Notifications(NotifyEffect::Add(n.clone())));
    }
    state.chrome.notifications.push(n);
    if state.chrome.notifications.len() > MAX_NOTIFICATIONS {
        state.chrome.notifications.remove(0);
        if saving {
            effects.push(Effect::Notifications(NotifyEffect::Prune));
        }
    }
    (state, effects)
}

/// The effect that writes a notice raised since `before` to the application log, if there is one.
/// The window calls it around each step, so that the update itself stays free of it.
pub fn log_new(before: &Option<Notice>, state: &State) -> Option<Effect> {
    let n = state
        .notice
        .as_ref()
        .filter(|n| Some(*n) != before.as_ref())?;
    Some(Effect::Notifications(NotifyEffect::Log(
        n.kind,
        n.text.clone(),
        state.wall_ms,
    )))
}

/// Keep every new notice in the list, and read the vault's stored list when a vault has just opened. Call after
/// each update with the notice from before it.
pub fn track(before: &Option<Notice>, state: State) -> (State, Vec<Effect>) {
    let (mut state, mut effects) = (state, Vec::new());
    let root = state.vault.as_ref().map(|v| v.root.clone());
    if root != state.chrome.notify_vault {
        if state.chrome.notify_vault.is_some() {
            state.chrome.notifications.clear();
            state.chrome.next_id = 1;
        }
        state.chrome.notify_loaded = false;
        if root.is_some() {
            effects.push(Effect::Notifications(NotifyEffect::Load));
        }
        state.chrome.notify_vault = root;
    }
    let Some(n) = state.notice.clone().filter(|n| Some(n) != before.as_ref()) else {
        return (state, effects);
    };
    let target = state
        .notice_link
        .as_ref()
        .filter(|(text, _)| *text == n.text)
        .map(|(_, uid)| NotifyTarget::Item(*uid));
    let (state, more) = push(state, (n.kind, n.text), target);
    effects.extend(more);
    (state, effects)
}

/// The stored list arrived: it comes first and what was raised meanwhile follows, and is stored now.
pub fn loaded(state: State, result: Result<Vec<Notification>, String>) -> (State, Vec<Effect>) {
    let mut state = state;
    let mut kept = result.unwrap_or_default();
    kept.retain(|n| n.at_ms >= state.wall_ms - MAX_AGE_MS);
    let mut next = kept.iter().map(|n| n.id).max().unwrap_or(0) + 1;
    let mut effects = vec![Effect::Notifications(NotifyEffect::Prune)];
    for mut n in std::mem::take(&mut state.chrome.notifications) {
        n.id = next;
        next += 1;
        effects.push(Effect::Notifications(NotifyEffect::Add(n.clone())));
        kept.push(n);
    }
    kept.sort_by_key(|n| (n.at_ms, n.id));
    (
        state.chrome.notifications,
        state.chrome.next_id,
        state.chrome.notify_loaded,
    ) = (kept, next, true);
    (state, effects)
}

fn stored(state: &State, op: NotifyEffect) -> Vec<Effect> {
    if persisting(state) {
        vec![Effect::Notifications(op)]
    } else {
        Vec::new()
    }
}

/// Handle what the user did in the panel.
pub fn update(state: State, msg: NotifyMsg) -> (State, Vec<Effect>) {
    let mut state = state;
    let effects = match msg {
        NotifyMsg::MarkRead(id, read) => {
            state
                .chrome
                .notifications
                .iter_mut()
                .filter(|n| n.id == id)
                .for_each(|n| n.read = read);
            stored(&state, NotifyEffect::SetRead(id, read))
        }
        NotifyMsg::MarkAll(read) => {
            state
                .chrome
                .notifications
                .iter_mut()
                .for_each(|n| n.read = read);
            stored(&state, NotifyEffect::SetAllRead(read))
        }
        NotifyMsg::Delete(id) => {
            state.chrome.notifications.retain(|n| n.id != id);
            stored(&state, NotifyEffect::Delete(id))
        }
        NotifyMsg::Clear => {
            state.chrome.notifications.clear();
            stored(&state, NotifyEffect::Clear)
        }
        NotifyMsg::Filter(f) => {
            state.chrome.notify_filter = f;
            Vec::new()
        }
        NotifyMsg::Open(id) => return open(state, id),
    };
    (state, effects)
}

fn open(state: State, id: u64) -> (State, Vec<Effect>) {
    let target = state
        .chrome
        .notifications
        .iter()
        .find(|n| n.id == id)
        .and_then(|n| n.target);
    let (mut state, effects) = update(state, NotifyMsg::MarkRead(id, true));
    state.chrome.menu = state.chrome.menu.filter(|m| *m != Menu::Notifications);
    let follow = match target {
        Some(NotifyTarget::Screen(s)) => Some(Msg::Navigate(s)),
        Some(NotifyTarget::Item(uid)) => Some(Msg::OpenItem(uid)),
        None => None,
    };
    if let Some(msg) = follow {
        state.replay = Some(Box::new(msg));
    }
    (state, effects)
}
