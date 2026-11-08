//! Item keys: short, human-readable names such as `STACK_1`, `FLASHCARD_500`, `NOTE_12` or
//! `QUESTION_3`, stored in an item's front matter (`key: FLASHCARD_500`) and never changed by a
//! rename. Pure: reading, writing and ordering keys; allocation lives in the store.

use std::cmp::Ordering;

use crate::query::Op;

/// The front-matter key that holds an item's key.
pub const KEY_FIELD: &str = "key";

/// Every prefix a key can have, with the kind word (as the index stores it) it belongs to.
pub const KEY_PREFIXES: [(&str, &str); 4] = [
    ("stack", "STACK"),
    ("card", "FLASHCARD"),
    ("note", "NOTE"),
    ("question", "QUESTION"),
];

/// The prefix of the keys of a kind (`card` gives `FLASHCARD`); `None` for a kind without keys.
pub fn key_prefix(kind: &str) -> Option<&'static str> {
    KEY_PREFIXES
        .iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, p)| *p)
}

/// The key with this prefix and number: `FLASHCARD_500`.
pub fn format_key(prefix: &str, n: u32) -> String {
    format!("{prefix}_{n}")
}

/// The prefix and number of a key, whatever its case; `None` for text that is not a key.
pub fn parse_key(text: &str) -> Option<(&'static str, u32)> {
    let (prefix, number) = text.trim().rsplit_once('_')?;
    let found = KEY_PREFIXES
        .iter()
        .find(|(_, p)| p.eq_ignore_ascii_case(prefix))?;
    let digits_only = !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit());
    let n = number
        .parse::<u32>()
        .ok()
        .filter(|n| digits_only && *n > 0)?;
    Some((found.1, n))
}

/// The key written the one way it is stored (upper case, no leading zeros); `None` if not a key.
pub fn normalize_key(text: &str) -> Option<String> {
    parse_key(text).map(|(p, n)| format_key(p, n))
}

/// Whether the text is a key.
pub fn is_key(text: &str) -> bool {
    parse_key(text).is_some()
}

/// The next free key of a prefix after the highest number among `taken` and `counter` (the
/// highest number ever handed out, so a deleted item's key is not given again).
pub fn next_key<'a>(
    prefix: &str,
    taken: impl IntoIterator<Item = &'a str>,
    counter: u32,
) -> String {
    let highest = taken
        .into_iter()
        .filter_map(parse_key)
        .filter(|(p, _)| *p == prefix)
        .map(|(_, n)| n)
        .fold(counter, u32::max);
    format_key(prefix, highest.saturating_add(1))
}

/// Keys in reading order: by prefix, then by number (`NOTE_2` before `NOTE_10`); text that is not a
/// key comes after the keys, by its letters.
pub fn compare_keys(a: &str, b: &str) -> Ordering {
    match (parse_key(a), parse_key(b)) {
        (Some(x), Some(y)) => x.cmp(&y),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => a.cmp(b),
    }
}

/// Whether an item's key satisfies `key op typed` in a query: case-insensitive, `NOTE_012` is
/// `NOTE_12`; an item without a key matches only `!=` and `!~`.
pub fn key_matches(op: Op, have: Option<&str>, typed: &str) -> bool {
    let norm = |t: &str| normalize_key(t).unwrap_or_else(|| t.trim().to_uppercase());
    let Some(have) = have else {
        return matches!(op, Op::Ne | Op::NotContains);
    };
    let (h, w) = (norm(have), norm(typed));
    match op {
        Op::Eq => h == w,
        Op::Ne => h != w,
        Op::Contains => h.contains(&w),
        Op::NotContains => !h.contains(&w),
        _ => false,
    }
}

/// Two optional keys in reading order (largest first when `descending`); a missing key comes last.
pub fn key_order(a: Option<&str>, b: Option<&str>, descending: bool) -> Ordering {
    match (a, b) {
        (Some(x), Some(y)) if descending => compare_keys(y, x),
        (Some(x), Some(y)) => compare_keys(x, y),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}
