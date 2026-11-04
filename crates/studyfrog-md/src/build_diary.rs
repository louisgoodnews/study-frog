//! Validating and constructing [`DiaryDoc`].

use std::collections::BTreeMap;

use studyfrog_core::civil::{format_date, parse_date};
use studyfrog_core::diary::RatingValue;
use studyfrog_core::meta::Meta;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;

use crate::doc::{DiaryDoc, Doc, RawEntry};
use crate::error::{ParseError, ParseErrorKind};
use crate::fence::Block;
use crate::parse::{fail, key_line};
use crate::raw::RawFront;
use crate::rfc3339::parse_rfc3339;
use crate::ulid::parse_ulid;

fn invalid(blocks: &[Block], key: &str, reason: &str) -> ParseError {
    let kind = ParseErrorKind::InvalidValue {
        key: key.to_owned(),
        reason: reason.to_owned(),
    };
    fail(key_line(blocks, key), kind)
}

fn text_ok(blocks: &[Block], key: &str, value: &str) -> Result<(), ParseError> {
    if value
        .chars()
        .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}'))
    {
        return Err(invalid(
            blocks,
            key,
            "must not contain control or line-break characters",
        ));
    }
    Ok(())
}

fn list(
    blocks: &[Block],
    key: &str,
    items: Option<Vec<String>>,
) -> Result<Vec<String>, ParseError> {
    let items = items.unwrap_or_default();
    for item in &items {
        if item.trim().is_empty() {
            return Err(invalid(blocks, key, "entries must not be empty"));
        }
        text_ok(blocks, key, item)?;
    }
    Ok(items)
}

fn uid_list(
    blocks: &[Block],
    key: &str,
    items: Option<Vec<String>>,
) -> Result<Vec<Uid>, ParseError> {
    let items = items.unwrap_or_default();
    let mut uids = Vec::with_capacity(items.len());
    for item in &items {
        let uid = parse_ulid(item)
            .ok_or_else(|| invalid(blocks, key, "must be a valid 26-character ULID"))?;
        uids.push(uid);
    }
    Ok(uids)
}

fn timestamp(
    blocks: &[Block],
    key: &str,
    value: Option<String>,
) -> Result<Option<UnixMs>, ParseError> {
    value
        .map(|v| {
            parse_rfc3339(&v)
                .ok_or_else(|| invalid(blocks, key, "not an RFC 3339 timestamp (years 0000-9999)"))
        })
        .transpose()
}

fn parse_standard_ratings(
    raw: &RawFront,
    blocks: &[Block],
) -> Result<BTreeMap<String, RatingValue>, ParseError> {
    let scales: [(&str, Option<u8>); 13] = [
        ("performance", raw.performance),
        ("success", raw.success),
        ("confidence", raw.confidence),
        ("effort", raw.effort),
        ("focus", raw.focus),
        ("energy", raw.energy),
        ("difficulty", raw.difficulty),
        ("satisfaction", raw.satisfaction),
        ("understanding", raw.understanding),
        ("mood", raw.mood),
        ("stress", raw.stress),
        ("motivation", raw.motivation),
        ("sleep", raw.sleep),
    ];
    let mut ratings = BTreeMap::new();
    for (name, val) in scales {
        if let Some(score) = val {
            if !(1..=5).contains(&score) {
                return Err(invalid(blocks, name, "rating must be between 1 and 5"));
            }
            ratings.insert(name.to_owned(), RatingValue::Int(i64::from(score)));
        }
    }
    Ok(ratings)
}

fn extract_custom_ratings(extra: &[RawEntry]) -> BTreeMap<String, RatingValue> {
    let mut custom = BTreeMap::new();
    for entry in extra {
        if let Some((k, v)) = entry.text.split_once(':') {
            let k = k.trim();
            let v = v.lines().next().unwrap_or(v).trim();
            if !k.is_empty() && k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                if let Ok(n) = v.parse::<i64>() {
                    custom.insert(k.to_owned(), RatingValue::Int(n));
                } else if !v.is_empty() && !v.starts_with('[') && !v.starts_with('{') {
                    let clean = v.trim_matches(|c| c == '"' || c == '\'');
                    custom.insert(k.to_owned(), RatingValue::Text(clean.to_owned()));
                }
            }
        }
    }
    custom
}

/// Construct a [`DiaryDoc`].
pub fn build_diary(
    mut raw: RawFront,
    extra: Vec<RawEntry>,
    body: &str,
    blocks: &[Block],
) -> Result<Doc, ParseError> {
    let id = raw
        .id
        .take()
        .ok_or_else(|| fail(None, ParseErrorKind::MissingKey("id")))?;
    let uid = parse_ulid(&id).ok_or_else(|| invalid(blocks, "id", "not a 26-character ULID"))?;
    let date_str = raw
        .date
        .take()
        .ok_or_else(|| fail(None, ParseErrorKind::MissingKey("date")))?;
    let date = parse_date(&date_str)
        .ok_or_else(|| invalid(blocks, "date", "must be an ISO date YYYY-MM-DD"))?;

    let custom_title = match raw.title.take() {
        Some(t) => {
            if t.trim().is_empty() {
                return Err(invalid(blocks, "title", "must not be empty"));
            }
            text_ok(blocks, "title", &t)?;
            Some(t)
        }
        None => None,
    };
    let title = custom_title
        .clone()
        .unwrap_or_else(|| format!("Diary: {}", format_date(date)));

    let meta = Meta {
        uid,
        title,
        tags: list(blocks, "tags", raw.tags.take())?,
        aliases: list(blocks, "aliases", raw.aliases.take())?,
        created: timestamp(blocks, "created", raw.created.take())?,
        updated: timestamp(blocks, "updated", raw.updated.take())?,
    };

    let mut ratings = parse_standard_ratings(&raw, blocks)?;
    for (k, v) in extract_custom_ratings(&extra) {
        ratings.entry(k).or_insert(v);
    }

    let runs = uid_list(blocks, "runs", raw.runs.take())?;
    let stacks = list(blocks, "stacks", raw.stacks.take())?;
    let items = uid_list(blocks, "items", raw.items.take())?;
    let exams = list(blocks, "exams", raw.exams.take())?;

    Ok(Doc::Diary(DiaryDoc {
        meta,
        date,
        custom_title,
        study_minutes: raw.study_minutes,
        ratings,
        runs,
        stacks,
        items,
        exams,
        body: body.to_owned(),
        extra,
    }))
}

/// Convert a [`DiaryDoc`] to a domain [`studyfrog_core::diary::DiaryEntry`].
pub fn diary_doc_to_entry(doc: &DiaryDoc) -> studyfrog_core::diary::DiaryEntry {
    studyfrog_core::diary::DiaryEntry {
        uid: doc.meta.uid,
        date: doc.date,
        title: doc.custom_title.clone(),
        created: doc.meta.created,
        updated: doc.meta.updated,
        study_minutes: doc.study_minutes,
        tags: doc.meta.tags.clone(),
        ratings: doc.ratings.clone(),
        runs: doc.runs.clone(),
        stacks: doc.stacks.clone(),
        items: doc.items.clone(),
        exams: doc.exams.clone(),
        content: doc.body.clone(),
    }
}

/// Convert a domain [`studyfrog_core::diary::DiaryEntry`] and unknown extra keys into a [`DiaryDoc`].
pub fn diary_entry_to_doc(
    entry: &studyfrog_core::diary::DiaryEntry,
    extra: Vec<RawEntry>,
) -> DiaryDoc {
    let title = entry
        .title
        .clone()
        .unwrap_or_else(|| format!("Diary: {}", format_date(entry.date)));
    DiaryDoc {
        meta: Meta {
            uid: entry.uid,
            title,
            tags: entry.tags.clone(),
            aliases: Vec::new(),
            created: entry.created,
            updated: entry.updated,
        },
        date: entry.date,
        custom_title: entry.title.clone(),
        study_minutes: entry.study_minutes,
        ratings: entry.ratings.clone(),
        runs: entry.runs.clone(),
        stacks: entry.stacks.clone(),
        items: entry.items.clone(),
        exams: entry.exams.clone(),
        body: entry.content.clone(),
        extra,
    }
}
