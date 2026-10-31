//! What a PDF is: its pages, title and outline.

use hayro::hayro_syntax::Pdf;

use crate::PdfBytes;
use crate::error::PdfError;
use crate::limits::{MAX_PAGES, check_size, guarded};

/// The size of a page as it is shown (rotation applied), in points (1/72 inch).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageSize {
    /// Width.
    pub width: f32,
    /// Height.
    pub height: f32,
}

/// A bookmark.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlineEntry {
    /// Text of the bookmark.
    pub title: String,
    /// The page it leads to (0-based), if known.
    pub page: Option<usize>,
    /// Nesting depth, 0 for the top level.
    pub level: u8,
}

/// Everything about a PDF that does not need a page to be read.
#[derive(Debug, Clone, PartialEq)]
pub struct PdfInfo {
    /// Number of pages.
    pub page_count: usize,
    /// Size of each page.
    pub pages: Vec<PageSize>,
    /// The document's title, if it has one.
    pub title: Option<String>,
    /// The author, if known.
    pub author: Option<String>,
    /// The subject, if known.
    pub subject: Option<String>,
    /// The bookmarks, flattened in reading order.
    pub outline: Vec<OutlineEntry>,
}

fn flatten(items: &[pdf_oxide::outline::OutlineItem], level: u8, out: &mut Vec<OutlineEntry>) {
    for item in items {
        let page = match &item.dest {
            Some(pdf_oxide::outline::Destination::PageIndex(i)) => Some(*i),
            _ => None,
        };
        out.push(OutlineEntry {
            title: item.title.trim().to_owned(),
            page,
            level,
        });
        if level < 8 {
            flatten(&item.children, level + 1, out);
        }
    }
}

fn info_field(doc: &lopdf::Document, key: &[u8]) -> Option<String> {
    let info = doc.trailer.get(b"Info").ok()?;
    let dict = match info {
        lopdf::Object::Reference(id) => doc.get_dictionary(*id).ok()?,
        lopdf::Object::Dictionary(d) => d,
        _ => return None,
    };
    let raw = dict.get(key).ok()?.as_str().ok()?;
    let text = text_string(raw);
    let text = text.trim().to_owned();
    (!text.is_empty()).then_some(text)
}

/// A PDF text string: UTF-16 with a byte order mark, else Latin-1.
pub fn text_string(raw: &[u8]) -> String {
    if let Some(rest) = raw.strip_prefix(&[0xFE, 0xFF]) {
        let units: Vec<u16> = rest
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| u16::from_be_bytes(*c))
            .collect();
        return String::from_utf16_lossy(&units);
    }
    raw.iter().map(|b| char::from(*b)).collect()
}

/// Read the structure of a PDF: page sizes, title, author and the outline.
pub fn pdf_info(bytes: &PdfBytes) -> Result<PdfInfo, PdfError> {
    check_size(bytes)?;
    let bytes = bytes.clone();
    guarded(move || {
        let pdf = Pdf::new(bytes.clone()).map_err(|e| match e {
            hayro::hayro_syntax::LoadPdfError::Decryption(_) => PdfError::Encrypted,
            other => PdfError::Invalid(format!("{other:?}")),
        })?;
        let pages = pdf.pages();
        if pages.len() > MAX_PAGES {
            return Err(PdfError::TooBig(format!(
                "it has more than {MAX_PAGES} pages"
            )));
        }
        let sizes = pages
            .iter()
            .map(|p| {
                let (w, h) = p.render_dimensions();
                PageSize {
                    width: w,
                    height: h,
                }
            })
            .collect::<Vec<_>>();
        let mut outline = Vec::new();
        if let Ok(oxide) = pdf_oxide::PdfDocument::from_bytes(bytes.as_ref().clone())
            && let Ok(Some(items)) = oxide.get_outline()
        {
            flatten(&items, 0, &mut outline);
        }
        let meta = lopdf::Document::load_mem(&bytes).ok();
        let field = |key: &[u8]| meta.as_ref().and_then(|d| info_field(d, key));
        Ok(PdfInfo {
            page_count: sizes.len(),
            pages: sizes,
            title: field(b"Title"),
            author: field(b"Author"),
            subject: field(b"Subject"),
            outline,
        })
    })
}
