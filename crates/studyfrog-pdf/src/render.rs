//! Drawing pages.

use std::sync::Arc;

use hayro::hayro_interpret::InterpreterSettings;
use hayro::hayro_syntax::Pdf;
use hayro::vello_cpu::color::palette::css::WHITE;
use hayro::{PixmapSettings, RenderCache, RenderSettings, render};

use crate::PdfBytes;
use crate::error::PdfError;
use crate::limits::{MAX_PAGE_POINTS, MAX_RENDER_PIXELS, check_size, guarded};

/// A drawn page: straight (not premultiplied) RGBA, row by row, on white.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pixels {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Four bytes per pixel.
    pub rgba: Vec<u8>,
}

/// A rectangle on a page in points, origin at the top left, y growing downwards.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageRect {
    /// Left edge.
    pub x: f32,
    /// Top edge.
    pub y: f32,
    /// Width.
    pub width: f32,
    /// Height.
    pub height: f32,
}

/// Draw a whole page at `scale` pixels per point (1.0 is 72 dpi, 2.0 is 144 dpi).
pub fn pdf_render_page(bytes: &PdfBytes, page: usize, scale: f32) -> Result<Pixels, PdfError> {
    check_size(bytes)?;
    if !(0.05..=16.0).contains(&scale) {
        return Err(PdfError::TooBig(
            "the zoom is outside 5 % to 1600 %".to_owned(),
        ));
    }
    let bytes: Arc<Vec<u8>> = bytes.clone();
    guarded(move || {
        let pdf = Pdf::new(bytes).map_err(|e| PdfError::Invalid(format!("{e:?}")))?;
        let pages = pdf.pages();
        let p = pages.get(page).ok_or(PdfError::NoSuchPage(page))?;
        let (w, h) = p.render_dimensions();
        if w > MAX_PAGE_POINTS || h > MAX_PAGE_POINTS || w <= 0.0 || h <= 0.0 {
            return Err(PdfError::TooBig(format!(
                "the page is {w:.0} by {h:.0} points"
            )));
        }
        let (pw, ph) = ((w * scale).ceil() as u64, (h * scale).ceil() as u64);
        if pw * ph > MAX_RENDER_PIXELS || pw > u64::from(u16::MAX) || ph > u64::from(u16::MAX) {
            return Err(PdfError::TooBig(format!(
                "drawing it at this zoom needs {} million pixels",
                pw * ph / 1_000_000
            )));
        }
        let settings = PixmapSettings {
            x_scale: scale,
            y_scale: scale,
            bg_color: WHITE,
        };
        let pix = render(
            p,
            &RenderCache::new(),
            &InterpreterSettings::default(),
            &RenderSettings::default(),
            &settings,
        );
        Ok(Pixels {
            width: u32::from(pix.width()),
            height: u32::from(pix.height()),
            rgba: pix.data_as_u8_slice().to_vec(),
        })
    })
}

/// Draw only `rect` of a page (a figure, a formula) at `scale`.
pub fn pdf_render_region(
    bytes: &PdfBytes,
    page: usize,
    rect: PageRect,
    scale: f32,
) -> Result<Pixels, PdfError> {
    let full = pdf_render_page(bytes, page, scale)?;
    let clamp = |v: f32, max: u32| ((v * scale).round().max(0.0) as u32).min(max);
    let (x0, y0) = (clamp(rect.x, full.width), clamp(rect.y, full.height));
    let (x1, y1) = (
        clamp(rect.x + rect.width, full.width),
        clamp(rect.y + rect.height, full.height),
    );
    if x1 <= x0 || y1 <= y0 {
        return Err(PdfError::Invalid("the selected area is empty".to_owned()));
    }
    let mut rgba = Vec::with_capacity(((x1 - x0) * (y1 - y0) * 4) as usize);
    for row in y0..y1 {
        let start = ((row * full.width + x0) * 4) as usize;
        rgba.extend_from_slice(&full.rgba[start..start + ((x1 - x0) * 4) as usize]);
    }
    Ok(Pixels {
        width: x1 - x0,
        height: y1 - y0,
        rgba,
    })
}

/// Encode drawn pixels as a PNG file.
pub fn pixels_to_png(p: &Pixels) -> Result<Vec<u8>, PdfError> {
    let img = image::RgbaImage::from_raw(p.width, p.height, p.rgba.clone())
        .ok_or_else(|| PdfError::Invalid("the pixel data is the wrong size".to_owned()))?;
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png)
        .map_err(|e| PdfError::Invalid(e.to_string()))?;
    Ok(out.into_inner())
}
