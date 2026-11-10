//! Detection and structured representation of alphabetical and custom-marked lists (STORY-0145).

use crate::blocks::{Block, ListItem, Span};

/// Detect if `text` begins with an alphabetical or parenthesized marker like `a. `, `(b) `, `c) `, `1. `.
pub fn alpha_marker(text: &str) -> Option<(&str, &str)> {
    let trimmed = text.trim_start();
    if trimmed.is_empty() {
        return None;
    }
    // Pattern: (letter or number) followed by whitespace, e.g. (a) , (1)
    if trimmed.starts_with('(')
        && let Some(close) = trimmed.find(')')
    {
        let inner = &trimmed[1..close];
        if !inner.is_empty()
            && inner.chars().all(|c| c.is_alphanumeric())
            && trimmed[close + 1..].starts_with(char::is_whitespace)
        {
            let marker = &trimmed[..=close];
            let rest = trimmed[close + 1..].trim_start();
            return Some((marker, rest));
        }
    }
    // Pattern: letter followed by '.' or ')', e.g. a. , B)
    let mut chars = trimmed.chars();
    if let (Some(first), Some(sep)) = (chars.next(), chars.next())
        && first.is_alphabetic()
        && (sep == '.' || sep == ')')
    {
        let next_char = chars.next();
        if next_char.is_none() || next_char.is_some_and(char::is_whitespace) {
            let len = first.len_utf8() + sep.len_utf8();
            let marker = &trimmed[..len];
            let rest = trimmed[len..].trim_start();
            return Some((marker, rest));
        }
    }
    None
}

/// Check if a line starts with an alphabetical marker.
pub fn is_alpha_marker(text: &str) -> bool {
    alpha_marker(text).is_some()
}

/// Convert a list of spans across lines into a structured `Block::List` if lines match list markers.
pub fn parse_alpha_list(spans: &[Span]) -> Option<Block> {
    if spans.is_empty() {
        return None;
    }

    // Split spans by newline characters into lines of spans
    let mut lines: Vec<Vec<Span>> = Vec::new();
    let mut current_line: Vec<Span> = Vec::new();

    for span in spans {
        let parts: Vec<&str> = span.text.split('\n').collect();
        for (idx, part) in parts.iter().enumerate() {
            if idx > 0 {
                lines.push(std::mem::take(&mut current_line));
            }
            if !part.is_empty() {
                current_line.push(Span {
                    text: (*part).to_owned(),
                    style: span.style,
                    link: span.link.clone(),
                });
            }
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        return None;
    }

    let mut items = Vec::with_capacity(lines.len());
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let first = &line[0];
        let (marker, rest) = alpha_marker(&first.text)?;
        let mut item_spans = Vec::new();
        if !rest.is_empty() {
            item_spans.push(Span {
                text: rest.to_owned(),
                style: first.style,
                link: first.link.clone(),
            });
        }
        item_spans.extend(line[1..].iter().cloned());

        items.push(ListItem {
            checked: None,
            marker: Some(marker.to_owned()),
            blocks: vec![Block::Paragraph(item_spans)],
        });
    }

    if items.is_empty() {
        return None;
    }

    Some(Block::List {
        ordered: true,
        start: 1,
        items,
    })
}
