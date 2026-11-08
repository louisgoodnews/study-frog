//! Image occlusion (STORY-0109, `docs/specs/image-occlusion.md`): rectangles hiding parts of a picture, kept
//! in the card's front matter as an `occlusion` block of plain text. The picture file is never changed.
//! Coordinates are fractions of the picture's width and height in ten-thousandths. Pure.
//!
//! ```yaml
//! occlusion:
//!   image: "assets/brain.png"
//!   mode: hide_all
//!   masks:
//!     - "0.1200 0.3400 0.2000 0.0500 Hippocampus"
//! ```

use crate::doc::{Doc, RawEntry};

/// The front-matter key.
pub const OCCLUSION_FIELD: &str = "occlusion";
/// One whole width or height.
pub const UNIT: u16 = 10_000;

/// One hidden rectangle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mask {
    /// Left edge, in ten-thousandths of the width.
    pub x: u16,
    /// Top edge, in ten-thousandths of the height.
    pub y: u16,
    /// Width.
    pub w: u16,
    /// Height.
    pub h: u16,
    /// What it hides (compared with a typed answer); may be empty.
    pub label: String,
}

/// How a run shows the masks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OcclusionMode {
    /// Every mask hides its part; the one asked is marked.
    #[default]
    HideAll,
    /// Only the mask asked hides its part.
    HideOne,
}

/// The masks of a card over one picture.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Occlusion {
    /// The picture, relative to the vault.
    pub image: String,
    /// Hide all or hide one.
    pub mode: OcclusionMode,
    /// The rectangles, in order (the order the run asks them).
    pub masks: Vec<Mask>,
}

/// A mask from fractions (clamped into the picture), with its label.
pub fn mask_from_unit((x, y, w, h): (f32, f32, f32, f32), label: &str) -> Mask {
    let q = |v: f32| (v.clamp(0.0, 1.0) * f32::from(UNIT)).round() as u16;
    let (x, y) = (q(x).min(UNIT - 1), q(y).min(UNIT - 1));
    Mask {
        x,
        y,
        w: q(w).clamp(1, UNIT - x),
        h: q(h).clamp(1, UNIT - y),
        label: label.trim().to_owned(),
    }
}

/// The mask as fractions `(x, y, w, h)`.
pub fn unit_of(m: &Mask) -> (f32, f32, f32, f32) {
    let f = |v: u16| f32::from(v) / f32::from(UNIT);
    (f(m.x), f(m.y), f(m.w), f(m.h))
}

fn fraction(v: u16) -> String {
    format!("{}.{:04}", v / UNIT, v % UNIT)
}

fn parse_fraction(t: &str) -> Option<u16> {
    let v: f64 = t.parse().ok()?;
    (0.0..=1.0)
        .contains(&v)
        .then(|| (v * f64::from(UNIT)).round() as u16)
}

fn quote(t: &str) -> String {
    format!("\"{}\"", t.replace('\\', "\\\\").replace('"', "\\\""))
}

fn unquote(t: &str) -> String {
    let t = t.trim();
    let Some(inner) = t.strip_prefix('"').and_then(|r| r.strip_suffix('"')) else {
        return t.to_owned();
    };
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        out.push(if c == '\\' {
            chars.next().unwrap_or('\\')
        } else {
            c
        });
    }
    out
}

fn mode_name(m: OcclusionMode) -> &'static str {
    match m {
        OcclusionMode::HideAll => "hide_all",
        OcclusionMode::HideOne => "hide_one",
    }
}

/// The block as written in the front matter (ending in a newline).
pub fn render_occlusion(o: &Occlusion) -> String {
    let mut out = format!(
        "{OCCLUSION_FIELD}:\n  image: {}\n  mode: {}\n  masks:\n",
        quote(&o.image),
        mode_name(o.mode)
    );
    for m in &o.masks {
        let line = format!(
            "{} {} {} {} {}",
            fraction(m.x),
            fraction(m.y),
            fraction(m.w),
            fraction(m.h),
            m.label
        );
        out.push_str(&format!("    - {}\n", quote(line.trim_end())));
    }
    out
}

fn parse_mask(item: &str) -> Option<Mask> {
    let text = unquote(item);
    let mut parts = text.splitn(5, ' ');
    let mut next = || parse_fraction(parts.next()?);
    let (x, y, w, h) = (next()?, next()?, next()?, next()?);
    let label = parts.next().unwrap_or("").trim().to_owned();
    let fits = w > 0
        && h > 0
        && u32::from(x) + u32::from(w) <= u32::from(UNIT)
        && u32::from(y) + u32::from(h) <= u32::from(UNIT);
    fits.then_some(Mask { x, y, w, h, label })
}

/// The block read back from its front-matter text; `None` when it is not a well-formed block.
pub fn parse_occlusion(text: &str) -> Option<Occlusion> {
    let mut o = Occlusion::default();
    for line in text.lines().skip(1) {
        let t = line.trim();
        if let Some(v) = t.strip_prefix("image:") {
            o.image = unquote(v);
        } else if let Some(v) = t.strip_prefix("mode:") {
            o.mode = match v.trim() {
                "hide_one" => OcclusionMode::HideOne,
                "hide_all" => OcclusionMode::HideAll,
                _ => return None,
            };
        } else if let Some(v) = t.strip_prefix("- ") {
            o.masks.push(parse_mask(v)?);
        } else if !(t.is_empty() || t == "masks:") {
            return None;
        }
    }
    (!o.image.is_empty()).then_some(o)
}

fn entries(doc: &Doc) -> Option<&Vec<RawEntry>> {
    match doc {
        Doc::Card(c) => Some(&c.extra),
        _ => None,
    }
}

/// The occlusion of a card, if it has a well-formed block.
pub fn occlusion_of(doc: &Doc) -> Option<Occlusion> {
    let entry = entries(doc)?.iter().find(|e| e.key == OCCLUSION_FIELD)?;
    parse_occlusion(&entry.text)
}

/// The card with its occlusion block replaced (or removed for `None`); other documents unchanged.
pub fn with_occlusion(doc: &Doc, o: Option<&Occlusion>) -> Doc {
    let mut out = doc.clone();
    if let Doc::Card(c) = &mut out {
        c.extra.retain(|e| e.key != OCCLUSION_FIELD);
        if let Some(o) = o {
            c.extra.push(RawEntry {
                key: OCCLUSION_FIELD.to_owned(),
                text: render_occlusion(o),
            });
        }
    }
    out
}

/// The picture paths `![..](path)` of a text, in order.
pub fn images_in(text: &str) -> Vec<String> {
    text.match_indices("![")
        .filter_map(|(i, _)| {
            let rest = &text[i..];
            let open = rest.find("](")?;
            let close = rest[open..].find(')')?;
            Some(rest[open + 2..open + close].trim().to_owned())
        })
        .collect()
}

/// The text without its pictures (what a run shows next to the masked picture).
pub fn without_images(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find("![") {
        let Some(end) = rest[i..]
            .find("](")
            .and_then(|o| rest[i + o..].find(')').map(|c| i + o + c + 1))
        else {
            break;
        };
        out.push_str(&rest[..i]);
        rest = &rest[end..];
    }
    out.push_str(rest);
    out.trim().to_owned()
}
