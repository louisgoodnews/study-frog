//! Notes made from PDF text: the text of pages or a selection becomes a Markdown note in a stack,
//! with a `source` key that leads back to the page.

use std::sync::Arc;

use studyfrog_pdf::doc::pdf_info;
use studyfrog_pdf::error::describe_pdf_error;
use studyfrog_pdf::text::{pdf_page_markdown, pdf_page_text};

use crate::create::IdSource;
use crate::create_more::{Created, NewItem, NewKind, create_item};
use crate::session::{ServiceError, VaultInfo, VaultSession};

/// Most pages one note may be made from.
pub const MAX_NOTE_PAGES: usize = 100;

/// What to make a note from.
#[derive(Debug, Clone, Copy)]
pub struct NoteRequest<'a> {
    /// Vault-relative path of the PDF.
    pub pdf: &'a str,
    /// First page (0-based).
    pub first: usize,
    /// Last page (0-based).
    pub last: usize,
    /// Text the user selected; used instead of the pages when given.
    pub selection: Option<&'a str>,
    /// The stack (vault-relative folder) the note goes into.
    pub folder: &'a str,
    /// Title; empty takes the PDF's name and the page.
    pub title: &'a str,
    /// Pictures (vault-relative paths) to put at the end of the note.
    pub pictures: &'a [String],
}

fn is_block(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with('#')
        || t.starts_with("- ")
        || t.starts_with("* ")
        || t.starts_with('|')
        || t.starts_with("```")
        || t.starts_with("> ")
        || t.split_once(". ")
            .is_some_and(|(n, _)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
}

/// Join the lines of one paragraph: a hyphen at the end of a line joins the word when the next line
/// goes on in lower case; otherwise lines are joined with a space.
fn join_lines(lines: &[&str]) -> String {
    let mut out = String::new();
    for line in lines.iter().map(|l| l.trim()).filter(|l| !l.is_empty()) {
        let hyphenated = out.ends_with('-') && line.chars().next().is_some_and(char::is_lowercase);
        if hyphenated {
            out.pop();
        } else if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(line);
    }
    out
}

/// Make running text out of text that was broken into lines by the page: paragraphs stay apart,
/// lines of a paragraph are joined, and headings, lists, quotes and tables are left alone.
pub fn reflow(text: &str) -> String {
    let normal = text.replace("\r\n", "\n");
    let paragraphs: Vec<String> = normal
        .split("\n\n")
        .map(|p| p.lines().collect::<Vec<_>>())
        .filter(|lines| lines.iter().any(|l| !l.trim().is_empty()))
        .map(|lines| {
            if lines.iter().any(|l| is_block(l)) {
                lines.join("\n")
            } else {
                join_lines(&lines)
            }
        })
        .collect();
    paragraphs.join("\n\n")
}

fn read_pdf(session: &VaultSession, rel: &str) -> Result<Arc<Vec<u8>>, ServiceError> {
    let clean = !rel.starts_with('/') && !rel.split('/').any(|p| p == ".." || p.is_empty());
    if !clean || !rel.to_lowercase().ends_with(".pdf") {
        return Err(ServiceError::Invalid(
            "only PDF files inside the vault can be used".to_owned(),
        ));
    }
    let data = std::fs::read(session.root.join(rel))
        .map_err(|e| ServiceError::Io(format!("cannot read {rel}: {e}")))?;
    Ok(Arc::new(data))
}

fn page_text(bytes: &Arc<Vec<u8>>, page: usize) -> Result<String, ServiceError> {
    let md = pdf_page_markdown(bytes, page)
        .map_err(|e| ServiceError::Invalid(describe_pdf_error(&e)))?;
    let text = reflow(&md);
    if !text.trim().is_empty() {
        return Ok(text);
    }
    let picture = pdf_page_text(bytes, page).is_ok_and(|t| t.image_only);
    Ok(if picture {
        "(this page is a picture without text; reading text from pictures is not part of StudyFrog yet)"
            .to_owned()
    } else {
        "(this page has no text)".to_owned()
    })
}

/// The `source` value that leads to a page (1-based in the text).
pub fn source_of(pdf: &str, page: usize) -> String {
    format!("{pdf}#page={}", page + 1)
}

/// Make the note.
pub fn note_from_pdf(
    session: &mut VaultSession,
    req: &NoteRequest<'_>,
    ids: IdSource,
) -> Result<(VaultInfo, Created), ServiceError> {
    let bytes = read_pdf(session, req.pdf)?;
    let info = pdf_info(&bytes).map_err(|e| ServiceError::Invalid(describe_pdf_error(&e)))?;
    let (first, last) = (req.first, req.last.max(req.first));
    if last >= info.page_count || last - first >= MAX_NOTE_PAGES {
        return Err(ServiceError::Invalid(format!(
            "choose pages 1 to {} of this PDF, at most {MAX_NOTE_PAGES} at a time",
            info.page_count
        )));
    }
    let body = match req.selection.map(reflow).filter(|t| !t.trim().is_empty()) {
        Some(t) => t,
        None => (first..=last)
            .map(|p| page_text(&bytes, p))
            .collect::<Result<Vec<_>, _>>()?
            .join("\n\n"),
    };
    let name = info
        .title
        .clone()
        .filter(|t| !t.trim().is_empty())
        .unwrap_or_else(|| req.pdf.to_owned());
    let pages = if first == last {
        format!("page {}", first + 1)
    } else {
        format!("pages {} to {}", first + 1, last + 1)
    };
    let title = if req.title.trim().is_empty() {
        format!("{name}, {pages}")
    } else {
        req.title.trim().to_owned()
    };
    let images: String = req
        .pictures
        .iter()
        .map(|p| format!("\n\n{}", picture_line(p)))
        .collect();
    let text = format!("From {name}, {pages}.\n\n{body}{images}");
    let source = source_of(req.pdf, first);
    let item = NewItem {
        kind: NewKind::Note,
        title: &title,
        text: &text,
        answer: "",
        tags: &[],
        parent: Some(req.folder),
        source: Some(&source),
    };
    create_item(session, &item, ids)
}

/// The Markdown that shows a picture of the vault.
fn picture_line(rel: &str) -> String {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    format!(
        "![{}]({rel})",
        name.rsplit_once('.').map_or(name, |(s, _)| s)
    )
}

/// What to make a card or a question from.
#[derive(Debug, Clone, Copy)]
pub struct ItemRequest<'a> {
    /// A card or a question.
    pub kind: NewKind,
    /// Vault-relative path of the PDF.
    pub pdf: &'a str,
    /// The page the text is from (0-based).
    pub page: usize,
    /// The front of a card or the prompt of a question.
    pub front: &'a str,
    /// The back of a card or the reference answer of a question.
    pub back: &'a str,
    /// The stack (vault-relative folder).
    pub folder: &'a str,
}

/// Make a card or a question with a source link to the page.
pub fn item_from_pdf(
    session: &mut VaultSession,
    req: &ItemRequest<'_>,
    ids: IdSource,
) -> Result<(VaultInfo, Created), ServiceError> {
    if !matches!(req.kind, NewKind::Card | NewKind::Question) {
        return Err(ServiceError::Invalid(
            "only cards and questions are made this way".to_owned(),
        ));
    }
    let bytes = read_pdf(session, req.pdf)?;
    let info = pdf_info(&bytes).map_err(|e| ServiceError::Invalid(describe_pdf_error(&e)))?;
    if req.page >= info.page_count {
        return Err(ServiceError::Invalid(format!(
            "that PDF has {} pages",
            info.page_count
        )));
    }
    let source = source_of(req.pdf, req.page);
    let item = NewItem {
        kind: req.kind,
        title: "",
        text: req.front,
        answer: req.back,
        tags: &[],
        parent: Some(req.folder),
        source: Some(&source),
    };
    create_item(session, &item, ids)
}
