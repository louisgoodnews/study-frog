//! [`Doc`] to text. Known keys are written in canonical order, defaults omitted; unknown keys
//! follow verbatim in their original relative order.

use std::borrow::Cow;

use studyfrog_core::meta::Meta;

use crate::doc::{Doc, Kind, RawEntry, kind_name};
use crate::error::RenderError;
use crate::raw::RawFront;
use crate::rfc3339::format_rfc3339;
use crate::sections::{join_card_body, split_card_body};
use crate::ulid::format_ulid;
use studyfrog_core::items::DEFAULT_MIN_SCORE;
use studyfrog_core::time::UnixMs;

fn non_empty(items: &[String]) -> Option<Vec<String>> {
    (!items.is_empty()).then(|| items.to_vec())
}

fn uids(list: &[studyfrog_core::uid::Uid]) -> Option<Vec<String>> {
    (!list.is_empty()).then(|| list.iter().map(|u| format_ulid(*u)).collect())
}

fn rating_score(
    ratings: &std::collections::BTreeMap<String, studyfrog_core::diary::RatingValue>,
    key: &str,
) -> Option<u8> {
    match ratings.get(key) {
        Some(studyfrog_core::diary::RatingValue::Int(n)) if (1..=5).contains(n) => Some(*n as u8),
        _ => None,
    }
}

fn raw_for_diary(d: &crate::doc::DiaryDoc) -> Result<RawFront, RenderError> {
    Ok(RawFront {
        id: Some(format_ulid(d.meta.uid)),
        kind: Some("diary".to_owned()),
        date: Some(studyfrog_core::civil::format_date(d.date)),
        title: d.custom_title.clone(),
        tags: non_empty(&d.meta.tags),
        aliases: non_empty(&d.meta.aliases),
        created: stamp(d.meta.created)?,
        updated: stamp(d.meta.updated)?,
        study_minutes: d.study_minutes,
        performance: rating_score(&d.ratings, "performance"),
        success: rating_score(&d.ratings, "success"),
        confidence: rating_score(&d.ratings, "confidence"),
        effort: rating_score(&d.ratings, "effort"),
        focus: rating_score(&d.ratings, "focus"),
        energy: rating_score(&d.ratings, "energy"),
        difficulty: rating_score(&d.ratings, "difficulty"),
        satisfaction: rating_score(&d.ratings, "satisfaction"),
        understanding: rating_score(&d.ratings, "understanding"),
        mood: rating_score(&d.ratings, "mood"),
        stress: rating_score(&d.ratings, "stress"),
        motivation: rating_score(&d.ratings, "motivation"),
        sleep: rating_score(&d.ratings, "sleep"),
        runs: uids(&d.runs),
        stacks: non_empty(&d.stacks),
        items: uids(&d.items),
        exams: non_empty(&d.exams),
        ..RawFront::default()
    })
}

fn stamp(at: Option<UnixMs>) -> Result<Option<String>, RenderError> {
    at.map(|t| format_rfc3339(t).ok_or(RenderError::TimestampOutOfRange))
        .transpose()
}

fn raw_for(kind: Kind, meta: &Meta) -> Result<RawFront, RenderError> {
    Ok(RawFront {
        id: Some(format_ulid(meta.uid)),
        kind: Some(kind_name(kind).to_owned()),
        title: Some(meta.title.clone()),
        tags: non_empty(&meta.tags),
        aliases: non_empty(&meta.aliases),
        created: stamp(meta.created)?,
        updated: stamp(meta.updated)?,
        ..RawFront::default()
    })
}

/// Front/back text must survive a split at the separator, or the file would not round-trip.
fn card_body(front: &str, back: &str) -> Result<String, RenderError> {
    let body = join_card_body(front, back);
    match split_card_body(&body) {
        Some((f, b)) if f == front && b == back => Ok(body),
        _ => Err(RenderError::AmbiguousSeparator),
    }
}

/// Front-matter, unknown keys and body of one document.
type Parts<'a> = (RawFront, &'a [RawEntry], Cow<'a, str>);

fn parts(doc: &Doc) -> Result<Parts<'_>, RenderError> {
    match doc {
        Doc::Stack(d) => {
            let raw = RawFront {
                description: Some(d.description.clone()).filter(|x| !x.is_empty()),
                ..raw_for(Kind::Stack, &d.meta)?
            };
            Ok((raw, &d.extra, Cow::Borrowed(&d.body)))
        }
        Doc::Note(d) => {
            let raw = RawFront {
                rehearse_title: d.rehearse_title.then_some(true),
                ..raw_for(Kind::Note, &d.meta)?
            };
            Ok((raw, &d.extra, Cow::Borrowed(&d.body)))
        }
        Doc::Card(d) => {
            let raw = RawFront {
                reversible: d.reversible.then_some(true),
                ..raw_for(Kind::Card, &d.meta)?
            };
            Ok((raw, &d.extra, Cow::Owned(card_body(&d.front, &d.back)?)))
        }
        Doc::Question(d) => {
            let raw = RawFront {
                accepted: non_empty(&d.accepted),
                rubric: non_empty(&d.rubric),
                min_score: Some(d.min_score)
                    .filter(|m| (*m - DEFAULT_MIN_SCORE).abs() > f32::EPSILON),
                ..raw_for(Kind::Question, &d.meta)?
            };
            Ok((
                raw,
                &d.extra,
                Cow::Owned(card_body(&d.prompt, &d.reference)?),
            ))
        }
        Doc::Diary(d) => {
            let raw = raw_for_diary(d)?;
            Ok((raw, &d.extra, Cow::Borrowed(&d.body)))
        }
    }
}

/// Render a document to file text.
pub fn render_doc(doc: &Doc) -> Result<String, RenderError> {
    let (raw, extra, body) = parts(doc)?;
    let yaml = serde_saphyr::to_string(&raw).map_err(|e| RenderError::Yaml(e.to_string()))?;
    let mut out = String::with_capacity(yaml.len() + body.len() + 16);
    out.push_str("---\n");
    out.push_str(&yaml);
    if !yaml.ends_with('\n') {
        out.push('\n');
    }
    for entry in extra {
        out.push_str(&entry.text);
        if !entry.text.ends_with('\n') {
            out.push('\n');
        }
    }
    out.push_str("---\n");
    out.push_str(&body);
    Ok(out)
}
