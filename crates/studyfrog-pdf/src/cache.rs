//! In-memory and disk LRU caching for rasterized PDF excerpts.

use std::collections::VecDeque;

use crate::PdfBytes;
use crate::crop::PdfCropAnchor;
use crate::error::PdfError;
use crate::render::{PageRect, Pixels, pdf_render_region, pixels_to_png};

/// A normalized lookup key identifying a rasterized crop excerpt.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CropCacheKey {
    /// Source document path or URI.
    pub path: String,
    /// 1-based page number.
    pub page: u32,
    /// X coordinate on the page (in points, rounded).
    pub x: u32,
    /// Y coordinate on the page (in points, rounded).
    pub y: u32,
    /// Width of the crop (in points, rounded).
    pub w: u32,
    /// Height of the crop (in points, rounded).
    pub h: u32,
    /// Scale factor multiplied by 1000 and rounded for deterministic comparison.
    pub scale_milli: u32,
}

/// A rasterized crop image buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RasterCrop {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// RGBA byte buffer (4 bytes per pixel).
    pub rgba: Vec<u8>,
}

/// Bounded LRU cache of rasterized PDF crops.
///
/// Front of `entries` is the most recently used (MRU) item; back is least recently used (LRU).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CropLruCache {
    /// Maximum number of entries retained.
    pub capacity: usize,
    /// Ordered entries (MRU at front, LRU at back).
    pub entries: VecDeque<(CropCacheKey, RasterCrop)>,
}

/// Creates a new `CropLruCache` with the given capacity.
pub fn new_crop_cache(capacity: usize) -> CropLruCache {
    CropLruCache {
        capacity,
        entries: VecDeque::with_capacity(capacity),
    }
}

/// Retrieves a reference to a cached raster crop, promoting it to the front of the LRU queue.
pub fn cache_get<'a>(cache: &'a mut CropLruCache, key: &CropCacheKey) -> Option<&'a RasterCrop> {
    let pos = cache.entries.iter().position(|(k, _)| k == key)?;
    if pos != 0 && let Some(entry) = cache.entries.remove(pos) {
        cache.entries.push_front(entry);
    }
    cache.entries.front().map(|(_, v)| v)
}

/// Inserts or updates an entry in the LRU cache, evicting the oldest if capacity is exceeded.
pub fn cache_put(cache: &mut CropLruCache, key: CropCacheKey, value: RasterCrop) {
    if let Some(pos) = cache.entries.iter().position(|(k, _)| k == &key) {
        cache.entries.remove(pos);
    }
    if cache.capacity == 0 {
        return;
    }
    while cache.entries.len() >= cache.capacity {
        cache.entries.pop_back();
    }
    cache.entries.push_front((key, value));
}

/// Returns the number of cached entries.
pub fn cache_len(cache: &CropLruCache) -> usize {
    cache.entries.len()
}

/// Clears all entries from the cache.
pub fn cache_clear(cache: &mut CropLruCache) {
    cache.entries.clear();
}

/// Constructs a `CropCacheKey` from a `PdfCropAnchor` and render scale.
pub fn make_crop_cache_key(anchor: &PdfCropAnchor, scale: f32) -> CropCacheKey {
    let x = if anchor.x.is_finite() && anchor.x > 0.0 {
        anchor.x.round() as u32
    } else {
        0
    };
    let y = if anchor.y.is_finite() && anchor.y > 0.0 {
        anchor.y.round() as u32
    } else {
        0
    };
    let w = if anchor.w.is_finite() && anchor.w > 0.0 {
        anchor.w.round() as u32
    } else {
        0
    };
    let h = if anchor.h.is_finite() && anchor.h > 0.0 {
        anchor.h.round() as u32
    } else {
        0
    };
    let scale_milli = if scale.is_finite() && scale > 0.0 {
        (scale * 1000.0).round() as u32
    } else {
        0
    };

    CropCacheKey {
        path: anchor.path.clone(),
        page: anchor.page,
        x,
        y,
        w,
        h,
        scale_milli,
    }
}

/// Generates a deterministic filesystem-safe filename for persisting a crop to disk.
pub fn crop_cache_filename(key: &CropCacheKey) -> String {
    let clean_path: String = key
        .path
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if clean_path.is_empty() {
        format!(
            "crop_p{}_x{}_y{}_w{}_h{}_s{}.png",
            key.page, key.x, key.y, key.w, key.h, key.scale_milli
        )
    } else {
        format!(
            "{clean_path}_p{}_x{}_y{}_w{}_h{}_s{}.png",
            key.page, key.x, key.y, key.w, key.h, key.scale_milli
        )
    }
}

/// Converts render `Pixels` into a `RasterCrop`.
pub fn raster_crop_from_pixels(pixels: Pixels) -> RasterCrop {
    RasterCrop {
        width: pixels.width,
        height: pixels.height,
        rgba: pixels.rgba,
    }
}

/// Converts a `RasterCrop` into render `Pixels`.
pub fn raster_crop_to_pixels(crop: &RasterCrop) -> Pixels {
    Pixels {
        width: crop.width,
        height: crop.height,
        rgba: crop.rgba.clone(),
    }
}

/// Encodes a `RasterCrop` into PNG image bytes for disk persistence.
pub fn raster_crop_to_png(crop: &RasterCrop) -> Result<Vec<u8>, PdfError> {
    pixels_to_png(&raster_crop_to_pixels(crop))
}

/// Decodes PNG image bytes from disk into a `RasterCrop`.
pub fn raster_crop_from_png(png_bytes: &[u8]) -> Result<RasterCrop, PdfError> {
    let img = image::load_from_memory(png_bytes)
        .map_err(|e| PdfError::Invalid(e.to_string()))?
        .to_rgba8();
    Ok(RasterCrop {
        width: img.width(),
        height: img.height(),
        rgba: img.into_raw(),
    })
}

/// Rasterizes a rectangular excerpt from PDF bytes according to the given `PdfCropAnchor`.
pub fn rasterize_crop(
    bytes: &PdfBytes,
    anchor: &PdfCropAnchor,
    scale: f32,
) -> Result<RasterCrop, PdfError> {
    if anchor.page == 0 {
        return Err(PdfError::NoSuchPage(0));
    }
    let rect = PageRect {
        x: anchor.x,
        y: anchor.y,
        width: anchor.w,
        height: anchor.h,
    };
    let pixels = pdf_render_region(bytes, (anchor.page - 1) as usize, rect, scale)?;
    Ok(raster_crop_from_pixels(pixels))
}

/// Retrieves a cached crop or rasterizes it on miss, updating the LRU cache.
pub fn cache_get_or_rasterize(
    cache: &mut CropLruCache,
    bytes: &PdfBytes,
    anchor: &PdfCropAnchor,
    scale: f32,
) -> Result<RasterCrop, PdfError> {
    let key = make_crop_cache_key(anchor, scale);
    if let Some(hit) = cache_get(cache, &key) {
        return Ok(hit.clone());
    }
    let raster = rasterize_crop(bytes, anchor, scale)?;
    cache_put(cache, key, raster.clone());
    Ok(raster)
}
