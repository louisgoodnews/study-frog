//! Standalone Markdown import: StudyFrog docs, Obsidian frontmatter, `???` / `Q:`-`A:` decks, and plain notes.

use studyfrog_core::plan::ItemKind;
use studyfrog_md::codeblock::advance_fence;
use studyfrog_md::parse::parse_doc;
use studyfrog_md::sections::split_card_body;

use crate::xfer_model::{Bundle, XItem, item};
use crate::xfer_obsidian_front::{count_unsupported, inline_tags, read_front, split_front};
use crate::xfer_plan::MAX_ITEMS;
use crate::xfer_target::from_studyfrog_doc;

/// What a standalone Markdown file can look like.
pub const MARKDOWN_LAYOUT: &str = "A .md file: a plain note, an Obsidian note with YAML frontmatter, a StudyFrog note/card/question, or a flashcard deck using ??? separators (with optional --- between cards) or Q:/A: pairs.";

fn stem_or_default(file_name: &str) -> String {
    let base = file_name
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(file_name)
        .trim();
    let lower = base.to_ascii_lowercase();
    let stem = if lower.ends_with(".markdown") {
        &base[..base.len() - 9]
    } else if lower.ends_with(".md") {
        &base[..base.len() - 3]
    } else {
        base
    };
    if stem.trim().is_empty() {
        "Note".to_owned()
    } else {
        stem.trim().to_owned()
    }
}

fn front_key(lines: &[String], key: &str) -> Option<String> {
    lines.iter().find_map(|line| {
        let (k, v) = line.split_once(':')?;
        let val = v.trim().trim_matches(['"', '\'']).trim();
        (k.trim().eq_ignore_ascii_case(key) && !val.is_empty()).then(|| val.to_owned())
    })
}

fn split_hr_sections(body: &str) -> Vec<String> {
    let (mut sections, mut cur, mut fence) = (Vec::new(), String::new(), None);
    for line in body.split_inclusive('\n') {
        let (next, in_code) = advance_fence(fence, line);
        fence = next;
        if !in_code && matches!(line.trim(), "---" | "***") {
            if !cur.trim().is_empty() {
                sections.push(cur.trim().to_owned());
            }
            cur.clear();
        } else {
            cur.push_str(line);
        }
    }
    if !cur.trim().is_empty() {
        sections.push(cur.trim().to_owned());
    }
    sections
}

fn split_all_sections(sections: &[String]) -> Option<Vec<(String, String)>> {
    if sections.is_empty() {
        return None;
    }
    let mut out = Vec::with_capacity(sections.len());
    for sec in sections {
        let (front, back) = split_card_body(sec)?;
        let (f, b) = (front.trim(), back.trim());
        if f.is_empty() || b.is_empty() {
            return None;
        }
        out.push((f.to_owned(), b.to_owned()));
    }
    Some(out)
}

fn push_qa(out: &mut Vec<(String, String)>, q: &str, a: &str) -> bool {
    let (qt, at) = (q.trim(), a.trim());
    if qt.is_empty() || at.is_empty() {
        return false;
    }
    out.push((qt.to_owned(), at.to_owned()));
    true
}

fn parse_qa_blocks(body: &str) -> Option<Vec<(String, String)>> {
    let (mut pairs, mut fence) = (Vec::new(), None);
    let (mut cur_q, mut cur_a, mut in_a) = (String::new(), String::new(), false);
    for line in body.lines() {
        let (next, in_code) = advance_fence(fence, line);
        fence = next;
        let t = line.trim_end();
        if !in_code && let Some(rest) = t.strip_prefix("Q:") {
            if !cur_q.is_empty() && !push_qa(&mut pairs, &cur_q, &cur_a) {
                return None;
            }
            (cur_q, in_a) = (rest.trim_start().to_owned(), false);
            cur_a.clear();
        } else if !in_code && let Some(rest) = t.strip_prefix("A:") {
            if cur_q.is_empty() || in_a {
                return None;
            }
            (cur_a, in_a) = (rest.trim_start().to_owned(), true);
        } else if in_a {
            cur_a.push('\n');
            cur_a.push_str(t);
        } else if !cur_q.is_empty() {
            cur_q.push('\n');
            cur_q.push_str(t);
        } else if !t.trim().is_empty() {
            return None;
        }
    }
    if cur_q.is_empty() || !push_qa(&mut pairs, &cur_q, &cur_a) {
        None
    } else {
        Some(pairs)
    }
}

fn make_cards(
    pairs: Vec<(String, String)>,
    kind: ItemKind,
    base: &XItem,
) -> Result<Vec<XItem>, String> {
    if pairs.len() > MAX_ITEMS {
        return Err(format!("the file holds more than {MAX_ITEMS} cards"));
    }
    let multiple = pairs.len() > 1;
    Ok(pairs
        .into_iter()
        .map(|(front, back)| {
            let mut it = base.clone();
            (it.kind, it.text, it.answer) = (kind, front, back);
            if multiple {
                it.title.clear();
            }
            it
        })
        .collect())
}

/// Parse a standalone Markdown file (StudyFrog, Obsidian, `???` / `Q:`-`A:` deck, or plain note).
pub fn from_markdown(file_name: &str, text: &str) -> Result<Bundle, String> {
    if text.trim().is_empty() {
        return Err("the Markdown file is empty".to_owned());
    }
    if let Ok(doc) = parse_doc(text) {
        return from_studyfrog_doc(file_name, &doc);
    }
    let (front_lines, body) = split_front(text);
    let kind_hint = front_key(&front_lines, "kind").map(|s| s.to_ascii_lowercase());
    if kind_hint.as_deref() == Some("stack") {
        return Err(format!(
            "{file_name} is a StudyFrog stack file (_stack.md); import its folder instead"
        ));
    }
    let front = read_front(&front_lines);
    let mut base = item(ItemKind::Note, "", body, "");
    base.title = front.title.unwrap_or_else(|| stem_or_default(file_name));
    base.tags = front.tags;
    for t in inline_tags(body) {
        if !base.tags.contains(&t) {
            base.tags.push(t);
        }
    }
    base.status = front_key(&front_lines, "status");
    base.priority = front_key(&front_lines, "priority");
    base.difficulty = front_key(&front_lines, "difficulty");
    base.extra = front
        .extra
        .into_iter()
        .filter(|(k, _)| !matches!(k.as_str(), "priority" | "difficulty"))
        .collect();
    let sections = split_hr_sections(body);
    let items = match kind_hint.as_deref() {
        Some("card" | "question") => {
            let kind = if kind_hint.as_deref() == Some("question") {
                ItemKind::Question
            } else {
                ItemKind::Card
            };
            let pairs = split_all_sections(&sections).ok_or_else(|| {
                "card or question Markdown needs a `???` separator line".to_owned()
            })?;
            make_cards(pairs, kind, &base)?
        }
        Some("note") => vec![base],
        _ => match split_all_sections(&sections).or_else(|| parse_qa_blocks(body)) {
            Some(pairs) => make_cards(pairs, ItemKind::Card, &base)?,
            None => vec![base],
        },
    };
    let mut warnings: Vec<String> = count_unsupported(body)
        .into_iter()
        .map(|(what, n)| format!("{n} {what} kept as text (StudyFrog does not render them)"))
        .collect();
    if !front.dropped.is_empty() {
        warnings.push(format!(
            "front-matter keys not kept: {}",
            front.dropped.join(", ")
        ));
    }
    Ok(Bundle {
        items,
        warnings,
        ..Bundle::default()
    })
}
