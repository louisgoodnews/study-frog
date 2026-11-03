//! Turning the pending text of the block parser into spans.

use crate::blocks::{Link, Span, Style};
use crate::blocks_build::Builder;
use crate::links::{Piece, split_wikilinks};

pub(crate) fn style(b: &Builder) -> Style {
    Style {
        bold: b.bold > 0,
        italic: b.italic > 0,
        strike: b.strike > 0,
        code: false,
        underline: b.underline > 0,
        color: b.colors.last().copied(),
    }
}

/// Turn the pending text into spans, splitting out wikilinks (never inside an explicit link).
pub(crate) fn flush_text(b: &mut Builder) {
    if b.text.is_empty() {
        return;
    }
    let text = std::mem::take(&mut b.text);
    let base = Span {
        text: String::new(),
        style: style(b),
        link: b.link.clone(),
    };
    if b.link.is_some() {
        b.spans.push(Span { text, ..base });
        return;
    }
    for piece in split_wikilinks(&text) {
        let span = match piece {
            Piece::Text(t) => Span {
                text: t.to_owned(),
                ..base.clone()
            },
            Piece::Link { target, shown } => Span {
                text: shown.to_owned(),
                link: Some(Link::Wiki(target.to_owned())),
                ..base.clone()
            },
        };
        b.spans.push(span);
    }
}

/// An image without a description still needs a span to carry its path.
pub(crate) fn end_image(b: &mut Builder) {
    flush_text(b);
    if !b.spans.last().is_some_and(|s| s.link == b.link) {
        let empty = Span {
            text: String::new(),
            style: style(b),
            link: b.link.clone(),
        };
        b.spans.push(empty);
    }
    b.link = None;
}
