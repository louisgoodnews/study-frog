//! Reading text: words with their places, plain text for search, and Markdown for notes.

use pdf_oxide::PdfDocument;
use pdf_oxide::converters::ConversionOptions;

use crate::PdfBytes;
use crate::error::PdfError;
use crate::limits::{FILE_TIME_LIMIT_SECS, MAX_PAGES, check_size, guarded, guarded_for};

/// A word and where it is, in points from the top left of the page.
#[derive(Debug, Clone, PartialEq)]
pub struct WordBox {
    /// The word.
    pub text: String,
    /// Left edge.
    pub x: f32,
    /// Top edge.
    pub y: f32,
    /// Width.
    pub width: f32,
    /// Height.
    pub height: f32,
}

/// The text of one page.
#[derive(Debug, Clone, PartialEq)]
pub struct PageText {
    /// Words in reading order.
    pub words: Vec<WordBox>,
    /// The page as plain text.
    pub text: String,
    /// The page has pictures but no text, such as a scanned page.
    pub image_only: bool,
}

fn open(bytes: &PdfBytes) -> Result<PdfDocument, PdfError> {
    PdfDocument::from_bytes(bytes.as_ref().clone()).map_err(|e| {
        let why = e.to_string();
        if why.to_lowercase().contains("encrypt") || why.to_lowercase().contains("password") {
            PdfError::Encrypted
        } else {
            PdfError::Invalid(why)
        }
    })
}

/// The six numbers of the map from the PDF's own space (points, y up, from the bottom left of the media box)
/// to the page as shown (points, y down, from the top left, with the page's rotation and crop applied).
fn page_transform(bytes: &PdfBytes, page: usize) -> Result<[f64; 6], PdfError> {
    let pdf = hayro::hayro_syntax::Pdf::new(bytes.clone())
        .map_err(|e| PdfError::Invalid(format!("{e:?}")))?;
    let pages = pdf.pages();
    let p = pages.get(page).ok_or(PdfError::NoSuchPage(page))?;
    Ok(p.initial_transform(true).as_coeffs())
}

/// A word box in the page as shown: the two corners of the box, mapped and put in order.
fn word_box(text: &str, (x, y, w, h): (f32, f32, f32, f32), m: [f64; 6]) -> WordBox {
    let map = |px: f32, py: f32| {
        let (px, py) = (f64::from(px), f64::from(py));
        (m[0] * px + m[2] * py + m[4], m[1] * px + m[3] * py + m[5])
    };
    let (a, b) = (map(x, y), map(x + w, y + h));
    let (left, right) = (a.0.min(b.0) as f32, a.0.max(b.0) as f32);
    let (top, bottom) = (a.1.min(b.1) as f32, a.1.max(b.1) as f32);
    WordBox {
        text: text.to_owned(),
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    }
}

fn count(doc: &PdfDocument, page: usize) -> Result<(), PdfError> {
    let n = doc
        .page_count()
        .map_err(|e| PdfError::Invalid(e.to_string()))?;
    if page >= n {
        return Err(PdfError::NoSuchPage(page));
    }
    Ok(())
}

/// Read one page: its words with their boxes, its text, and whether it is only a picture.
pub fn pdf_page_text(bytes: &PdfBytes, page: usize) -> Result<PageText, PdfError> {
    check_size(bytes)?;
    let bytes = bytes.clone();
    guarded(move || {
        let doc = open(&bytes)?;
        count(&doc, page)?;
        let to_page = page_transform(&bytes, page)?;
        let words: Vec<WordBox> = doc
            .extract_words(page)
            .map_err(|e| PdfError::Invalid(e.to_string()))?
            .into_iter()
            .filter(|w| !w.text.trim().is_empty())
            .map(|w| {
                word_box(
                    w.text.trim(),
                    (w.bbox.x, w.bbox.y, w.bbox.width, w.bbox.height),
                    to_page,
                )
            })
            .collect();
        let text = doc
            .extract_text(page)
            .map_err(|e| PdfError::Invalid(e.to_string()))?;
        let image_only = text.trim().is_empty()
            && doc
                .extract_images(page)
                .map(|i| !i.is_empty())
                .unwrap_or(false);
        Ok(PageText {
            words,
            text,
            image_only,
        })
    })
}

/// The plain text of every page, for searching. Pages that cannot be read give an empty string.
pub fn pdf_all_text(bytes: &PdfBytes) -> Result<Vec<PageText>, PdfError> {
    check_size(bytes)?;
    let bytes = bytes.clone();
    guarded_for(FILE_TIME_LIMIT_SECS, move || {
        let doc = open(&bytes)?;
        let n = doc
            .page_count()
            .map_err(|e| PdfError::Invalid(e.to_string()))?;
        if n > MAX_PAGES {
            return Err(PdfError::TooBig(format!(
                "it has more than {MAX_PAGES} pages"
            )));
        }
        Ok((0..n)
            .map(|i| {
                let text = doc.extract_text(i).unwrap_or_default();
                let image_only = text.trim().is_empty()
                    && doc
                        .extract_images(i)
                        .map(|v| !v.is_empty())
                        .unwrap_or(false);
                PageText {
                    words: Vec::new(),
                    text,
                    image_only,
                }
            })
            .collect())
    })
}

/// A page as Markdown, with headings recovered from font sizes where possible.
pub fn pdf_page_markdown(bytes: &PdfBytes, page: usize) -> Result<String, PdfError> {
    check_size(bytes)?;
    let bytes = bytes.clone();
    guarded(move || {
        let doc = open(&bytes)?;
        count(&doc, page)?;
        let options = ConversionOptions {
            detect_headings: true,
            ..ConversionOptions::default()
        };
        doc.to_markdown(page, &options)
            .map_err(|e| PdfError::Invalid(e.to_string()))
    })
}
