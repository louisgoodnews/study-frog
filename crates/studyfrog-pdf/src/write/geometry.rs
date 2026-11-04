//! Page size, margins, and geometric measurements for layout and PDF output.

/// Standard and custom page dimensions in PDF points (1/72 inch).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PageSize {
    /// ISO A4: 595.28 x 841.89 points (210 x 297 mm).
    A4,
    /// US Letter: 612.0 x 792.0 points (8.5 x 11.0 in).
    Letter,
    /// Custom page width and height in points.
    Custom(f32, f32),
}

/// Margin insets from the four outer edges of a page in points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Margins {
    /// Distance from the top edge.
    pub top: f32,
    /// Distance from the bottom edge.
    pub bottom: f32,
    /// Distance from the left edge.
    pub left: f32,
    /// Distance from the right edge.
    pub right: f32,
}

/// Full geometric description of a document's pages.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageGeometry {
    /// Paper size.
    pub size: PageSize,
    /// Margins around the content area.
    pub margins: Margins,
    /// Distance reserved for the running header at top.
    pub header_height: f32,
    /// Distance reserved for the running footer at bottom.
    pub footer_height: f32,
}

/// Standard 54 pt (0.75 in / 19.05 mm) margins.
pub fn default_margins() -> Margins {
    Margins {
        top: 54.0,
        bottom: 54.0,
        left: 54.0,
        right: 54.0,
    }
}

/// The width and height of a given [`PageSize`] in points.
pub fn page_size_dimensions(size: PageSize) -> (f32, f32) {
    match size {
        PageSize::A4 => (595.28, 841.89),
        PageSize::Letter => (612.0, 792.0),
        PageSize::Custom(w, h) => (w, h),
    }
}

/// Standard document geometry for the specified page size with 54 pt margins.
pub fn default_geometry(size: PageSize) -> PageGeometry {
    PageGeometry {
        size,
        margins: default_margins(),
        header_height: 36.0,
        footer_height: 36.0,
    }
}

/// Width of the printable content column in points.
pub fn printable_width(geom: &PageGeometry) -> f32 {
    let (w, _) = page_size_dimensions(geom.size);
    (w - geom.margins.left - geom.margins.right).max(10.0)
}

/// Height of the printable content area (between header and footer) in points.
pub fn printable_height(geom: &PageGeometry) -> f32 {
    let (_, h) = page_size_dimensions(geom.size);
    let top = geom.margins.top;
    let bottom = geom.margins.bottom;
    (h - top - bottom).max(10.0)
}

/// Content bounding box as `(left, top, width, height)` in points from page top-left.
pub fn content_box(geom: &PageGeometry) -> (f32, f32, f32, f32) {
    (
        geom.margins.left,
        geom.margins.top,
        printable_width(geom),
        printable_height(geom),
    )
}
