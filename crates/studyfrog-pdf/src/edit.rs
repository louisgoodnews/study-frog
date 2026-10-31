//! Changing pages: build a new PDF from pages of others (reorder, delete, rotate, merge, split) and
//! edit the document properties. The original bytes are never changed; the result is always new.

use lopdf::{Dictionary, Object, StringFormat};
use pdf_oxide::editor::DocumentEditor;

use crate::PdfBytes;
use crate::error::PdfError;
use crate::limits::{FILE_TIME_LIMIT_SECS, MAX_PAGES, check_size, guarded_for};

/// One page of the new file: a page of one of the source files, turned clockwise by `rotate` degrees.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageSpec {
    /// Which of the source files (an index into the list given to [`pdf_build`]).
    pub source: usize,
    /// The page of that file (0-based).
    pub page: usize,
    /// Extra clockwise rotation: 0, 90, 180 or 270.
    pub rotate: i32,
}

/// Document properties; `None` leaves a property as it is.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Properties {
    /// Title.
    pub title: Option<String>,
    /// Author.
    pub author: Option<String>,
    /// Subject.
    pub subject: Option<String>,
}

fn bad(e: impl std::fmt::Display) -> PdfError {
    PdfError::Invalid(e.to_string())
}

/// Remove objects nothing refers to any more (pages that were left out) and write the file.
fn tidy(bytes: &[u8]) -> Result<Vec<u8>, PdfError> {
    let mut doc = lopdf::Document::load_mem(bytes).map_err(bad)?;
    doc.prune_objects();
    let mut out = Vec::new();
    doc.save_to(&mut out).map_err(bad)?;
    Ok(out)
}

/// Build a new PDF from `plan`, taking pages from `sources`. Pages of one source that follow each other
/// are taken together; the order of the plan is the order of the result.
pub fn pdf_build(sources: &[PdfBytes], plan: &[PageSpec]) -> Result<Vec<u8>, PdfError> {
    if plan.is_empty() {
        return Err(PdfError::Invalid("there are no pages to write".to_owned()));
    }
    if plan.len() > MAX_PAGES {
        return Err(PdfError::TooBig(format!(
            "the result would have more than {MAX_PAGES} pages"
        )));
    }
    for s in sources {
        check_size(s)?;
    }
    if plan
        .iter()
        .any(|p| p.source >= sources.len() || p.rotate % 90 != 0)
    {
        return Err(PdfError::Invalid(
            "the page plan refers to a file or an angle that does not exist".to_owned(),
        ));
    }
    let (sources, plan) = (sources.to_vec(), plan.to_vec());
    guarded_for(FILE_TIME_LIMIT_SECS, move || {
        let mut merged: Option<DocumentEditor> = None;
        let mut at = 0;
        while at < plan.len() {
            let source = plan[at].source;
            let run: Vec<usize> = plan[at..]
                .iter()
                .take_while(|p| p.source == source)
                .map(|p| p.page)
                .collect();
            let mut editor =
                DocumentEditor::from_bytes(sources[source].as_ref().clone()).map_err(bad)?;
            let chunk = editor.extract_pages_to_bytes(&run).map_err(bad)?;
            at += run.len();
            match &mut merged {
                None => merged = Some(DocumentEditor::from_bytes(chunk).map_err(bad)?),
                Some(m) => {
                    m.merge_from_bytes(&chunk).map_err(bad)?;
                }
            }
        }
        let mut editor =
            merged.ok_or_else(|| PdfError::Invalid("there are no pages to write".to_owned()))?;
        for (i, spec) in plan.iter().enumerate().filter(|(_, p)| p.rotate != 0) {
            editor
                .rotate_page_by(i, spec.rotate.rem_euclid(360))
                .map_err(bad)?;
        }
        tidy(&editor.save_to_bytes().map_err(bad)?)
    })
}

/// Ranges such as `1-3, 5` (pages counted from 1) as 0-based inclusive ranges; each must lie inside
/// `1..=page_count`.
pub fn parse_page_ranges(text: &str, page_count: usize) -> Result<Vec<(usize, usize)>, String> {
    let mut out = Vec::new();
    for part in text.split(',').map(str::trim).filter(|p| !p.is_empty()) {
        let (a, b) = part
            .split_once('-')
            .map_or((part, part), |(a, b)| (a.trim(), b.trim()));
        let num = |s: &str| {
            s.parse::<usize>()
                .map_err(|_| format!("\"{part}\" is not a page or a range like 2-5"))
        };
        let (a, b) = (num(a)?, num(b)?);
        if a == 0 || b < a || b > page_count {
            return Err(format!("\"{part}\" is outside the pages 1 to {page_count}"));
        }
        out.push((a - 1, b - 1));
    }
    if out.is_empty() {
        return Err("Type the pages to keep, for example 1-3, 7".to_owned());
    }
    Ok(out)
}

/// Pages `ranges` of one file, in order, as a new PDF.
pub fn pdf_extract_ranges(
    bytes: &PdfBytes,
    ranges: &[(usize, usize)],
) -> Result<Vec<u8>, PdfError> {
    let plan: Vec<PageSpec> = ranges
        .iter()
        .flat_map(|(a, b)| {
            (*a..=*b).map(|page| PageSpec {
                source: 0,
                page,
                rotate: 0,
            })
        })
        .collect();
    pdf_build(std::slice::from_ref(bytes), &plan)
}

/// A PDF text string for `s`.
pub fn pdf_text_object(s: &str) -> Object {
    if s.is_ascii() {
        return Object::String(s.as_bytes().to_vec(), StringFormat::Literal);
    }
    let mut v = vec![0xFE, 0xFF];
    v.extend(s.encode_utf16().flat_map(u16::to_be_bytes));
    Object::String(v, StringFormat::Literal)
}

/// Set title, author or subject; the rest of the file is kept as it is.
pub fn pdf_set_properties(bytes: &PdfBytes, props: &Properties) -> Result<Vec<u8>, PdfError> {
    check_size(bytes)?;
    let (bytes, props) = (bytes.clone(), props.clone());
    guarded_for(FILE_TIME_LIMIT_SECS, move || {
        let mut doc = lopdf::Document::load_mem(&bytes).map_err(bad)?;
        write_properties(&mut doc, &props)?;
        let mut out = Vec::new();
        doc.save_to(&mut out).map_err(bad)?;
        Ok(out)
    })
}

/// Put `props` into the document's Info dictionary, creating it when there is none.
pub fn write_properties(doc: &mut lopdf::Document, props: &Properties) -> Result<(), PdfError> {
    let id = match doc
        .trailer
        .get(b"Info")
        .ok()
        .and_then(|o| o.as_reference().ok())
    {
        Some(id) => id,
        None => {
            let id = doc.add_object(Dictionary::new());
            doc.trailer.set("Info", Object::Reference(id));
            id
        }
    };
    let info = doc.get_dictionary_mut(id).map_err(bad)?;
    for (key, value) in [
        ("Title", &props.title),
        ("Author", &props.author),
        ("Subject", &props.subject),
    ] {
        if let Some(v) = value {
            info.set(key, pdf_text_object(v.trim()));
        }
    }
    Ok(())
}
