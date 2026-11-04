//! Validated [`RawFront`] plus body to [`Doc`].

use studyfrog_core::items::DEFAULT_MIN_SCORE;
use studyfrog_core::meta::Meta;
use studyfrog_core::time::UnixMs;

use crate::doc::{CardDoc, Doc, Kind, NoteDoc, QuestionDoc, RawEntry, StackDoc};
use crate::error::{ParseError, ParseErrorKind};
use crate::fence::Block;
use crate::parse::{fail, key_line};
use crate::raw::RawFront;
use crate::rfc3339::parse_rfc3339;
use crate::sections::split_card_body;
use crate::ulid::parse_ulid;

/// Characters that would break single-line values: controls and Unicode line breaks.
fn is_unsafe(c: char) -> bool {
    c.is_control() || matches!(c, '\u{2028}' | '\u{2029}')
}

fn invalid(blocks: &[Block], key: &str, reason: &str) -> ParseError {
    let kind = ParseErrorKind::InvalidValue {
        key: key.to_owned(),
        reason: reason.to_owned(),
    };
    fail(key_line(blocks, key), kind)
}

fn text_ok(blocks: &[Block], key: &str, value: &str) -> Result<(), ParseError> {
    if value.chars().any(is_unsafe) {
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

fn build_meta(raw: &mut RawFront, blocks: &[Block]) -> Result<Meta, ParseError> {
    let id = raw
        .id
        .take()
        .ok_or_else(|| fail(None, ParseErrorKind::MissingKey("id")))?;
    let uid = parse_ulid(&id).ok_or_else(|| invalid(blocks, "id", "not a 26-character ULID"))?;
    let title = raw
        .title
        .take()
        .ok_or_else(|| fail(None, ParseErrorKind::MissingKey("title")))?;
    if title.trim().is_empty() {
        return Err(invalid(blocks, "title", "must not be empty"));
    }
    text_ok(blocks, "title", &title)?;
    Ok(Meta {
        uid,
        title,
        tags: list(blocks, "tags", raw.tags.take())?,
        aliases: list(blocks, "aliases", raw.aliases.take())?,
        created: timestamp(blocks, "created", raw.created.take())?,
        updated: timestamp(blocks, "updated", raw.updated.take())?,
    })
}

fn unit_interval(blocks: &[Block], key: &str, value: f32, open: bool) -> Result<f32, ParseError> {
    let ok = if open {
        value > 0.0 && value < 1.0
    } else {
        (0.0..=1.0).contains(&value)
    };
    if ok {
        Ok(value)
    } else {
        Err(invalid(blocks, key, "must be between 0 and 1"))
    }
}

fn build_stack(meta: Meta, raw: RawFront, extra: Vec<RawEntry>, body: &str) -> Doc {
    Doc::Stack(StackDoc {
        meta,
        description: raw.description.unwrap_or_default(),
        body: body.to_owned(),
        extra,
    })
}

fn build_question(
    meta: Meta,
    raw: RawFront,
    extra: Vec<RawEntry>,
    body: &str,
    blocks: &[Block],
) -> Result<Doc, ParseError> {
    let (prompt, reference) =
        split_card_body(body).ok_or_else(|| fail(None, ParseErrorKind::MissingSeparator))?;
    let min_score = unit_interval(
        blocks,
        "min_score",
        raw.min_score.unwrap_or(DEFAULT_MIN_SCORE),
        false,
    )?;
    Ok(Doc::Question(QuestionDoc {
        meta,
        prompt: prompt.to_owned(),
        reference: reference.to_owned(),
        accepted: list(blocks, "accepted", raw.accepted)?,
        rubric: list(blocks, "rubric", raw.rubric)?,
        min_score,
        extra,
    }))
}

/// Combine validated front-matter and the body into a document of `kind`.
pub fn build_doc(
    kind: Kind,
    mut raw: RawFront,
    extra: Vec<RawEntry>,
    body: &str,
    blocks: &[Block],
) -> Result<Doc, ParseError> {
    if kind == Kind::Diary {
        return crate::build_diary::build_diary(raw, extra, body, blocks);
    }
    let meta = build_meta(&mut raw, blocks)?;
    match kind {
        Kind::Stack => Ok(build_stack(meta, raw, extra, body)),
        Kind::Question => build_question(meta, raw, extra, body, blocks),
        Kind::Note => Ok(Doc::Note(NoteDoc {
            meta,
            rehearse_title: raw.rehearse_title.unwrap_or(false),
            body: body.to_owned(),
            extra,
        })),
        Kind::Card => {
            let (front, back) = split_card_body(body)
                .ok_or_else(|| fail(None, ParseErrorKind::MissingSeparator))?;
            let reversible = raw.reversible.unwrap_or(false);
            Ok(Doc::Card(CardDoc {
                meta,
                reversible,
                front: front.to_owned(),
                back: back.to_owned(),
                extra,
            }))
        }
        Kind::Diary => unreachable!(),
    }
}
