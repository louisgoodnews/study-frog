//! PDF files as plain data (ADR 0010). `hayro` draws pages, `pdf_oxide` reads text, pictures and the
//! outline, and `pdf_oxide` and `lopdf` change pages and properties. Nothing outside this crate sees
//! those libraries. Every call works on the bytes of a file, runs under the limits in [`limits`] and
//! cannot take the program down: a panic or a hang inside a library comes back as an error.

pub mod annotate;
pub mod crop;
pub mod doc;
pub mod edit;
pub mod error;
pub mod images;
pub mod limits;
pub mod links;
pub mod render;
pub mod text;
pub mod write;

pub use crop::{
    PdfCropAnchor, compute_crop_rect, crop_rect_area, format_pdf_crop_uri, is_valid_crop_anchor,
    is_valid_crop_rect, normalized_to_points, parse_pdf_crop_uri, points_to_normalized,
};

use std::sync::Arc;

/// The bytes of a PDF file, shared cheaply between calls and threads.
pub type PdfBytes = Arc<Vec<u8>>;
