//! Tables, underline and colour for the block parser: the pipe tables of GitHub Markdown, and the two small HTML
//! forms (`<u>` and a coloured `<span>`) that Markdown has no syntax for.

use pulldown_cmark::{Alignment, Tag, TagEnd};

use crate::blocks::{Align, Block, Row};
use crate::blocks_build::{Builder, push_block};
use crate::blocks_text::flush_text;
use crate::hue::{Hue, hue_of_tag};

/// A table being read.
#[derive(Default)]
pub(crate) struct TableAcc {
    aligns: Vec<Align>,
    header: Row,
    rows: Vec<Row>,
    row: Row,
}

fn align(a: Alignment) -> Align {
    match a {
        Alignment::None => Align::None,
        Alignment::Left => Align::Left,
        Alignment::Center => Align::Center,
        Alignment::Right => Align::Right,
    }
}

/// A tag of a table starts.
pub(crate) fn start(b: &mut Builder, tag: &Tag) {
    match tag {
        Tag::Table(aligns) => {
            let aligns = aligns.iter().copied().map(align).collect();
            b.table = Some(TableAcc {
                aligns,
                ..TableAcc::default()
            });
        }
        Tag::TableHead | Tag::TableRow => {
            if let Some(t) = &mut b.table {
                t.row.clear();
            }
        }
        _ => {}
    }
}

/// A tag of a table ends.
pub(crate) fn end(b: &mut Builder, tag: TagEnd) {
    match tag {
        TagEnd::TableCell => {
            flush_text(b);
            let cell = std::mem::take(&mut b.spans);
            if let Some(t) = &mut b.table {
                t.row.push(cell);
            }
        }
        TagEnd::TableHead => {
            if let Some(t) = &mut b.table {
                t.header = std::mem::take(&mut t.row);
            }
        }
        TagEnd::TableRow => {
            if let Some(t) = &mut b.table {
                let row = std::mem::take(&mut t.row);
                t.rows.push(row);
            }
        }
        TagEnd::Table => {
            if let Some(t) = b.table.take() {
                let TableAcc {
                    aligns,
                    header,
                    rows,
                    ..
                } = t;
                push_block(
                    b,
                    Block::Table {
                        aligns,
                        header,
                        rows,
                    },
                );
            }
        }
        _ => {}
    }
}

/// What an inline HTML tag means for the text that follows.
enum Format {
    Underline(bool),
    Colour(Option<Hue>),
}

fn format_of(b: &Builder, tag: &str) -> Option<Format> {
    match tag.trim() {
        "<u>" => Some(Format::Underline(true)),
        "</u>" if b.underline > 0 => Some(Format::Underline(false)),
        "</span>" if !b.colors.is_empty() => Some(Format::Colour(None)),
        other => hue_of_tag(other).map(|h| Format::Colour(Some(h))),
    }
}

/// Inline HTML: `<u>`, `</u>` and a coloured span are formatting; anything else is shown as typed.
pub(crate) fn html(b: &mut Builder, tag: &str) {
    let tag = tag.trim_end_matches('\n');
    let Some(format) = format_of(b, tag) else {
        b.text.push_str(tag);
        return;
    };
    flush_text(b);
    match format {
        Format::Underline(true) => b.underline += 1,
        Format::Underline(false) => b.underline -= 1,
        Format::Colour(Some(hue)) => b.colors.push(hue),
        Format::Colour(None) => {
            b.colors.pop();
        }
    }
}
