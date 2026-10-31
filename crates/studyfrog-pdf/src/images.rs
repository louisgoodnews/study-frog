//! Pictures that are part of a page.

use crate::PdfBytes;
use crate::error::PdfError;
use crate::limits::{check_size, guarded};

/// A picture on a page, as a PNG file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageImage {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// The picture as a PNG file.
    pub png: Vec<u8>,
}

/// Most pictures returned for one page.
pub const MAX_IMAGES_PER_PAGE: usize = 50;

/// The pictures on one page, as PNG files. A picture that cannot be converted is skipped.
pub fn pdf_page_images(bytes: &PdfBytes, page: usize) -> Result<Vec<PageImage>, PdfError> {
    check_size(bytes)?;
    let bytes = bytes.clone();
    guarded(move || {
        let doc = pdf_oxide::PdfDocument::from_bytes(bytes.as_ref().clone())
            .map_err(|e| PdfError::Invalid(e.to_string()))?;
        let n = doc
            .page_count()
            .map_err(|e| PdfError::Invalid(e.to_string()))?;
        if page >= n {
            return Err(PdfError::NoSuchPage(page));
        }
        let found = doc
            .extract_images(page)
            .map_err(|e| PdfError::Invalid(e.to_string()))?;
        Ok(found
            .iter()
            .take(MAX_IMAGES_PER_PAGE)
            .filter_map(|i| {
                i.to_png_bytes().ok().map(|png| PageImage {
                    width: i.width(),
                    height: i.height(),
                    png,
                })
            })
            .collect())
    })
}
