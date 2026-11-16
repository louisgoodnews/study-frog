//! Coordinate highlight overlay and animated pulsing for PDF visual excerpts.
//!
//! Renders semi-transparent focus rectangles over cropped page regions.

use eframe::egui::{self, Rect, Vec2};
use studyfrog_pdf::crop::{PdfCropAnchor, normalized_to_points};

/// Compute animated pulsing alpha over time for the coordinate highlight overlay.
///
/// Pulses between 0.3 and 0.9 for the first 3.0 seconds, then settles into a steady 0.4 overlay.
pub fn highlight_alpha(start_time: f64, now: f64) -> f32 {
    let elapsed = (now - start_time).max(0.0);
    if elapsed < 3.0 {
        let pulse = (elapsed * 6.0).sin().abs() as f32;
        0.3 + 0.6 * pulse
    } else {
        0.4
    }
}

/// Converts crop anchor coordinates into point-based coordinates on the page.
pub fn anchor_to_points(anchor: &PdfCropAnchor, page_bounds: (f32, f32)) -> (f32, f32, f32, f32) {
    if anchor.x <= 1.0
        && anchor.y <= 1.0
        && anchor.w <= 1.0
        && anchor.h <= 1.0
        && page_bounds.0 > 1.0
        && page_bounds.1 > 1.0
    {
        normalized_to_points((anchor.x, anchor.y, anchor.w, anchor.h), page_bounds)
    } else {
        (anchor.x, anchor.y, anchor.w, anchor.h)
    }
}

/// Computes the logical screen rectangle for a crop anchor within a rendered page.
pub fn crop_to_screen_rect(
    anchor: &PdfCropAnchor,
    page_bounds: (f32, f32),
    page_rect: Rect,
    logical: f32,
) -> Rect {
    let (x, y, w, h) = anchor_to_points(anchor, page_bounds);
    let min = page_rect.min + Vec2::new(x, y) * logical;
    let size = Vec2::new(w, h) * logical;
    Rect::from_min_size(min, size)
}

/// Paint a pulsing coordinate highlight overlay over a crop region.
pub fn paint_crop_highlight(
    ui: &egui::Ui,
    anchor: &PdfCropAnchor,
    page_bounds: (f32, f32),
    page_rect: Rect,
    logical: f32,
    (flash_start, now): (f64, f64),
) {
    let rect = crop_to_screen_rect(anchor, page_bounds, page_rect, logical);
    let alpha = highlight_alpha(flash_start, now);
    let p = crate::theme::palette_of(ui);
    ui.painter()
        .rect_filled(rect, 2.0, p.accent.gamma_multiply(alpha * 0.5));
    let stroke = egui::Stroke::new(2.5, p.accent.gamma_multiply((alpha + 0.2).min(1.0)));
    ui.painter()
        .rect_stroke(rect, 2.0, stroke, egui::StrokeKind::Outside);
    if (now - flash_start).max(0.0) < 3.0 {
        ui.ctx().request_repaint();
    }
}
