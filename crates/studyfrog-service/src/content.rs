//! What the rehearsal screen shows for an item, read from its file on demand.

use std::fs;

use studyfrog_core::review::Rating;
use studyfrog_core::uid::Uid;
use studyfrog_md::doc::{Doc, doc_meta};
use studyfrog_md::parse::parse_doc;
use studyfrog_store::overview::item_path;
use studyfrog_store::schedule::load_reviews;

use crate::session::{ServiceError, VaultSession};

/// An item reduced to what is asked and what is answered, trimmed for display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemContent {
    /// The item's title.
    pub title: String,
    /// What is shown first: a card's front, a question's prompt, a note's title.
    pub prompt: String,
    /// What is revealed: a card's back, a question's reference answer, a note's body.
    pub answer: String,
    /// The rating of the item's latest review before now; `None` for an item never reviewed.
    pub previous: Option<Rating>,
    /// Where the item was made from (such as `bio/cells.pdf#page=3`), if it says.
    pub source: Option<String>,
    /// The item's colour (a palette word or `#RRGGBB`), if it has one.
    pub color: Option<String>,
    /// What a typed answer is compared with when it is not the whole answer (a cloze gap's words).
    pub expected: Option<String>,
}

fn content_of(doc: &Doc) -> ItemContent {
    let title = doc_meta(doc).title.clone();
    let shown = |text: &str| unescape_newlines(text.trim_end());
    match doc {
        Doc::Card(c) => ItemContent {
            title,
            prompt: shown(&c.front),
            answer: shown(&c.back),
            previous: None,
            source: None,
            color: None,
            expected: None,
        },
        Doc::Question(q) => ItemContent {
            title,
            prompt: shown(&q.prompt),
            answer: shown(&q.reference),
            previous: None,
            source: None,
            color: None,
            expected: None,
        },
        Doc::Note(n) => ItemContent {
            prompt: title.clone(),
            title,
            answer: shown(&n.body),
            previous: None,
            source: None,
            color: None,
            expected: None,
        },
        Doc::Stack(s) => ItemContent {
            prompt: title.clone(),
            title,
            answer: shown(&s.description),
            previous: None,
            source: None,
            color: None,
            expected: None,
        },
        Doc::Diary(d) => ItemContent {
            prompt: title.clone(),
            title,
            answer: shown(&d.body),
            previous: None,
            source: None,
            color: None,
            expected: None,
        },
    }
}

/// Read and parse the item's file. The path comes from the index, so a renamed file is found.
pub fn load_doc(session: &VaultSession, uid: Uid) -> Result<Doc, ServiceError> {
    let rel = item_path(&session.conn, uid)
        .map_err(ServiceError::Store)?
        .ok_or_else(|| ServiceError::Invalid("that item is no longer in the vault".to_owned()))?;
    let path = session.root.join(&rel);
    let text = fs::read_to_string(&path).map_err(|e| ServiceError::Io(format!("{rel}: {e}")))?;
    parse_doc(&text).map_err(|e| ServiceError::Invalid(format!("{rel}: {e}")))
}

/// What the rehearsal screen shows for an item.
pub fn load_item_content(session: &VaultSession, uid: Uid) -> Result<ItemContent, ServiceError> {
    let reviews = load_reviews(&session.conn, uid).map_err(ServiceError::Store)?;
    let previous = reviews.last().map(|r| r.rating);
    load_doc(session, uid).map(|doc| {
        let content = ItemContent {
            previous,
            source: studyfrog_md::extra::extra_value(&doc, "source"),
            color: studyfrog_md::extra::extra_value(&doc, studyfrog_core::item_color::COLOR_FIELD)
                .and_then(|c| studyfrog_core::item_color::normalize_color(&c)),
            ..content_of(&doc)
        };
        crate::content_cloze::with_cloze(&doc, content, reviews.len())
    })
}

/// Convert literal `\r\n` and `\n` sequences outside `$...$` math spans into newline characters.
pub fn unescape_newlines(text: &str) -> String {
    if !text.contains("\\n") && !text.contains("\\r\\n") {
        return text.to_owned();
    }
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut in_math = false;
    while let Some(c) = chars.next() {
        if c == '$' {
            in_math = !in_math;
            out.push(c);
        } else if !in_math && c == '\\' {
            if chars.next_if_eq(&'n').is_some() {
                out.push('\n');
            } else if chars.next_if_eq(&'r').is_some() {
                if chars.next_if_eq(&'\\').is_some() && chars.next_if_eq(&'n').is_some() {
                    out.push('\n');
                } else {
                    out.push_str("\\r");
                }
            } else {
                out.push('\\');
                if let Some(next) = chars.next() {
                    out.push(next);
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Repair existing card files in `root` that have literal `\n` sequences or 60-char truncated titles.
pub fn repair_imported_cards(conn: &rusqlite::Connection, root: &std::path::Path) {
    let Ok(files) = studyfrog_store::vault::list_markdown(root) else {
        return;
    };
    for f in files {
        let Ok(raw) = fs::read_to_string(&f.abs) else {
            continue;
        };
        let Ok(Doc::Card(mut c)) = parse_doc(&raw) else {
            continue;
        };
        let (front, back) = (unescape_newlines(&c.front), unescape_newlines(&c.back));
        let full_title = crate::create::default_card_title(&front);
        let truncated = c.meta.title.chars().count() == 60
            && full_title.chars().count() > 60
            && full_title.starts_with(&c.meta.title)
            && crate::create::check_title(&full_title).is_ok();
        if front != c.front || back != c.back || truncated {
            c.front = front;
            c.back = back;
            if truncated {
                c.meta.title = full_title;
            }
            if let Ok(rendered) = studyfrog_md::render::render_doc(&Doc::Card(c))
                && crate::create::write_new(&f.abs, &rendered).is_ok()
            {
                let _ = conn.execute("UPDATE file SET size = -1 WHERE path = ?1", [&f.rel]);
            }
        }
    }
}
