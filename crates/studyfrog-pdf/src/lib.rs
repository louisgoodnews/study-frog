//! PDF files as plain data (ADR 0010). `hayro` draws pages, `pdf_oxide` reads text, pictures and the
//! outline, and `pdf_oxide` and `lopdf` change pages and properties. Nothing outside this crate sees
//! those libraries. Every call works on the bytes of a file, runs under the limits in [`limits`] and
//! cannot take the program down: a panic or a hang inside a library comes back as an error.

pub mod annotate;
pub mod cache;
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

pub use cache::{
    CropCacheKey, CropLruCache, RasterCrop, cache_clear, cache_get, cache_get_or_rasterize,
    cache_len, cache_put, crop_cache_filename, make_crop_cache_key, new_crop_cache,
    raster_crop_from_pixels, raster_crop_from_png, raster_crop_to_pixels, raster_crop_to_png,
    rasterize_crop,
};
pub use crop::{
    PdfCropAnchor, compute_crop_rect, crop_rect_area, format_pdf_crop_uri, is_valid_crop_anchor,
    is_valid_crop_rect, normalized_to_points, parse_pdf_crop_uri, points_to_normalized,
};

use std::sync::Arc;

/// The bytes of a PDF file, shared cheaply between calls and threads.
pub type PdfBytes = Arc<Vec<u8>>;
