//! Text to [`Doc`]. Entry point is [`parse_doc`].

use serde::Deserialize;

use crate::build::build_doc;
use crate::doc::{Doc, Kind, RawEntry, kind_from_name, known_keys};
use crate::error::{ParseError, ParseErrorKind};
use crate::fence::{Block, split_blocks, split_front_matter};
use crate::raw::RawFront;

/// The YAML region starts on line 2 of the file (line 1 is the opening fence).
pub const YAML_FIRST_LINE: usize = 2;

/// Parse a whole file. Never panics; every failure is a typed [`ParseError`].
pub fn parse_doc(text: &str) -> Result<Doc, ParseError> {
    let split =
        split_front_matter(text).ok_or_else(|| fail(None, ParseErrorKind::MissingFrontMatter))?;
    let blocks = split_blocks(split.yaml);
    let kind = read_kind(&blocks)?;
    let known = known_keys(kind);
    let (known_blocks, extra) = partition(&blocks, &known);
    let raw = decode(&known_blocks)?;
    build_doc(kind, raw, extra, split.body, &blocks)
}

/// Build a parse error.
pub fn fail(line: Option<usize>, kind: ParseErrorKind) -> ParseError {
    ParseError { line, kind }
}

/// File line of a front-matter key, when present.
pub fn key_line(blocks: &[Block], key: &str) -> Option<usize> {
    blocks
        .iter()
        .find(|b| b.key == key)
        .map(|b| b.line + YAML_FIRST_LINE - 1)
}

#[derive(Deserialize)]
struct KindOnly {
    kind: String,
}

fn read_kind(blocks: &[Block]) -> Result<Kind, ParseError> {
    let block = blocks
        .iter()
        .find(|b| b.key == "kind")
        .ok_or_else(|| fail(None, ParseErrorKind::MissingKey("kind")))?;
    let line = Some(block.line + YAML_FIRST_LINE - 1);
    let parsed: KindOnly = serde_saphyr::from_str(block.text).map_err(|e| {
        fail(
            line,
            ParseErrorKind::InvalidYaml(e.without_snippet().to_string()),
        )
    })?;
    kind_from_name(&parsed.kind).ok_or_else(|| fail(line, ParseErrorKind::UnknownKind(parsed.kind)))
}

fn partition<'a>(blocks: &[Block<'a>], known: &[&str]) -> (Vec<Block<'a>>, Vec<RawEntry>) {
    let (mine, rest): (Vec<Block<'a>>, Vec<Block<'a>>) =
        blocks.iter().copied().partition(|b| known.contains(&b.key));
    let extra = rest
        .iter()
        .map(|b| RawEntry {
            key: b.key.to_owned(),
            text: b.text.to_owned(),
        })
        .collect();
    (mine, extra)
}

/// Decode only the blocks this kind understands, so a malformed foreign key cannot fail the file.
fn decode(known: &[Block]) -> Result<RawFront, ParseError> {
    let text: String = known.iter().map(|b| b.text).collect();
    if text.trim().is_empty() {
        return Ok(RawFront::default());
    }
    serde_saphyr::from_str(&text).map_err(|e| {
        let line = e
            .location()
            .and_then(|l| file_line(known, l.line() as usize));
        fail(
            line,
            ParseErrorKind::InvalidYaml(e.without_snippet().to_string()),
        )
    })
}

/// Map a 1-based line of the concatenated known blocks back to a file line.
fn file_line(known: &[Block], relative: usize) -> Option<usize> {
    let mut before = 0;
    for b in known {
        let lines = b.text.split_inclusive('\n').count();
        if relative <= before + lines {
            return Some(b.line + (relative - before - 1) + YAML_FIRST_LINE - 1);
        }
        before += lines;
    }
    None
}
