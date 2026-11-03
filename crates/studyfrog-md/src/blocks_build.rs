//! The Markdown event walker behind [`parse_blocks`](crate::blocks::parse_blocks).

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

use crate::blocks::{Block, Link, ListItem, Span, Style};
use crate::blocks_heading::level;
use crate::blocks_table::TableAcc;
use crate::blocks_text::{end_image, flush_text, style};
use crate::hue::Hue;

enum Frame {
    Root(Vec<Block>),
    Quote(Vec<Block>),
    List {
        ordered: bool,
        start: u64,
        items: Vec<ListItem>,
    },
    Item(ListItem),
}

#[derive(Default)]
pub(crate) struct Builder {
    stack: Vec<Frame>,
    pub(crate) spans: Vec<Span>,
    pub(crate) text: String,
    pub(crate) bold: u32,
    pub(crate) italic: u32,
    pub(crate) strike: u32,
    pub(crate) underline: u32,
    pub(crate) colors: Vec<Hue>,
    pub(crate) link: Option<Link>,
    code: Option<(String, String)>,
    heading: Option<u8>,
    pub(crate) table: Option<TableAcc>,
}

pub(crate) fn push_block(b: &mut Builder, block: Block) {
    match b.stack.last_mut() {
        Some(Frame::Root(v) | Frame::Quote(v)) => v.push(block),
        Some(Frame::Item(item)) => item.blocks.push(block),
        _ => {}
    }
}

/// A paragraph that is only `[TOC]` asks for the table of contents.
fn is_toc(spans: &[Span]) -> bool {
    matches!(spans, [s] if s.link.is_none() && s.text.trim().eq_ignore_ascii_case("[toc]"))
}

/// Pending inline content that was not wrapped in a paragraph (tight list items) becomes one.
fn flush_paragraph(b: &mut Builder) {
    flush_text(b);
    if !b.spans.is_empty() {
        let spans = std::mem::take(&mut b.spans);
        let block = if is_toc(&spans) {
            Block::Toc
        } else {
            Block::Paragraph(spans)
        };
        push_block(b, block);
    }
}

fn start(b: &mut Builder, tag: Tag) {
    match tag {
        Tag::Heading { level: l, .. } => b.heading = Some(level(l)),
        Tag::BlockQuote(_) => {
            flush_paragraph(b);
            b.stack.push(Frame::Quote(Vec::new()));
        }
        Tag::List(first) => {
            flush_paragraph(b);
            b.stack.push(Frame::List {
                ordered: first.is_some(),
                start: first.unwrap_or(1),
                items: Vec::new(),
            });
        }
        Tag::Item => b.stack.push(Frame::Item(ListItem::default())),
        Tag::CodeBlock(kind) => {
            flush_paragraph(b);
            let lang = match kind {
                CodeBlockKind::Fenced(info) => info.to_string(),
                CodeBlockKind::Indented => String::new(),
            };
            b.code = Some((lang, String::new()));
        }
        Tag::Emphasis => (flush_text(b), b.italic += 1).1,
        Tag::Strong => (flush_text(b), b.bold += 1).1,
        Tag::Strikethrough => (flush_text(b), b.strike += 1).1,
        Tag::Image { dest_url, .. } => {
            flush_text(b);
            b.link = Some(Link::Image(dest_url.to_string()));
        }
        Tag::Link { dest_url, .. } => {
            flush_text(b);
            b.link = Some(Link::Url(dest_url.to_string()));
        }
        other => crate::blocks_table::start(b, &other),
    }
}

fn end(b: &mut Builder, tag: TagEnd) {
    match tag {
        TagEnd::Paragraph => flush_paragraph(b),
        TagEnd::Heading(_) => {
            flush_text(b);
            let spans = std::mem::take(&mut b.spans);
            let level = b.heading.take().unwrap_or(1);
            push_block(b, Block::Heading { level, spans });
        }
        TagEnd::BlockQuote(_) => {
            flush_paragraph(b);
            if let Some(Frame::Quote(v)) = b.stack.pop() {
                push_block(b, Block::Quote(v));
            }
        }
        TagEnd::List(_) => {
            if let Some(Frame::List {
                ordered,
                start,
                items,
            }) = b.stack.pop()
            {
                push_block(
                    b,
                    Block::List {
                        ordered,
                        start,
                        items,
                    },
                );
            }
        }
        TagEnd::Item => {
            flush_paragraph(b);
            if let Some(Frame::Item(item)) = b.stack.pop()
                && let Some(Frame::List { items, .. }) = b.stack.last_mut()
            {
                items.push(item);
            }
        }
        TagEnd::CodeBlock => {
            if let Some((lang, text)) = b.code.take() {
                let tokens = crate::highlight::highlight(&lang, &text);
                push_block(b, Block::Code { lang, text, tokens });
            }
        }
        TagEnd::Emphasis => (flush_text(b), b.italic = b.italic.saturating_sub(1)).1,
        TagEnd::Strong => (flush_text(b), b.bold = b.bold.saturating_sub(1)).1,
        TagEnd::Strikethrough => (flush_text(b), b.strike = b.strike.saturating_sub(1)).1,
        TagEnd::Image => end_image(b),
        TagEnd::Link => {
            flush_text(b);
            b.link = None;
        }
        other => crate::blocks_table::end(b, other),
    }
}

fn event(b: &mut Builder, ev: Event) {
    match ev {
        Event::Start(tag) => start(b, tag),
        Event::End(tag) => end(b, tag),
        Event::Text(t) => match &mut b.code {
            Some((_, code)) => code.push_str(&t),
            None => b.text.push_str(&t),
        },
        Event::Code(c) => {
            flush_text(b);
            b.spans.push(Span {
                text: c.to_string(),
                style: Style {
                    code: true,
                    ..style(b)
                },
                link: b.link.clone(),
            });
        }
        Event::SoftBreak => b.text.push(' '),
        Event::HardBreak => b.text.push('\n'),
        Event::Html(t) | Event::InlineHtml(t) => crate::blocks_table::html(b, &t),
        Event::Rule => {
            flush_paragraph(b);
            push_block(b, Block::Rule);
        }
        Event::TaskListMarker(done) => {
            if let Some(Frame::Item(item)) = b.stack.last_mut() {
                item.checked = Some(done);
            }
        }
        _ => {}
    }
}

/// Walk the events of `markdown` into blocks.
pub(crate) fn build(markdown: &str) -> Vec<Block> {
    let mut b = Builder {
        stack: vec![Frame::Root(Vec::new())],
        ..Builder::default()
    };
    let options =
        Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS | Options::ENABLE_TABLES;
    for ev in Parser::new_ext(markdown, options) {
        event(&mut b, ev);
    }
    flush_paragraph(&mut b);
    match b.stack.into_iter().next() {
        Some(Frame::Root(blocks)) => blocks,
        _ => Vec::new(),
    }
}
