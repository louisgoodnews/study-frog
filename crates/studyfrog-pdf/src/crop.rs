//! Rectangular crop selection and coordinate schema for PDF visual excerpts.
//!
//! Stores bounding box coordinates in URL anchor / frontmatter format:
//! `path/to/doc.pdf#page=N&bbox=x,y,w,h`

/// A rectangular crop anchor pointing to a specific page and bounding box inside a PDF.
///
/// Bounding box coordinates represent normalized [0.0, 1.0] or point-based coordinates on the page.
#[derive(Debug, Clone, PartialEq)]
pub struct PdfCropAnchor {
    /// Relative or absolute path to the PDF file.
    pub path: String,
    /// 1-based page number.
    pub page: u32,
    /// X coordinate (left edge) of the bounding box.
    pub x: f32,
    /// Y coordinate (top edge) of the bounding box.
    pub y: f32,
    /// Width of the bounding box.
    pub w: f32,
    /// Height of the bounding box.
    pub h: f32,
}

fn format_coord(v: f32) -> String {
    if v.fract() == 0.0 {
        format!("{v:.1}")
    } else {
        format!("{v}")
    }
}

/// Formats a `PdfCropAnchor` into a frontmatter URI anchor string.
///
/// Format: `path#page=N&bbox=x,y,w,h`
pub fn format_pdf_crop_uri(anchor: &PdfCropAnchor) -> String {
    format!(
        "{}#page={}&bbox={},{},{},{}",
        anchor.path,
        anchor.page,
        format_coord(anchor.x),
        format_coord(anchor.y),
        format_coord(anchor.w),
        format_coord(anchor.h),
    )
}

/// Parses a PDF crop URI into a `PdfCropAnchor`.
///
/// Supports URI formats such as:
/// - `path/to/doc.pdf#page=84&bbox=150.0,220.0,500.0,480.0`
/// - `source: path/to/doc.pdf#page=84&bbox=150.0,220.0,500.0,480.0`
/// - With or without surrounding quotes.
pub fn parse_pdf_crop_uri(uri: &str) -> Option<PdfCropAnchor> {
    let mut s = uri.trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s = s[1..s.len() - 1].trim();
    }
    if let Some(rest) = s.strip_prefix("source:") {
        s = rest.trim();
    }
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s = s[1..s.len() - 1].trim();
    }

    let (path, fragment) = s.split_once('#')?;
    let path = path.trim();
    if path.is_empty() {
        return None;
    }

    let mut page = None;
    let mut bbox = None;

    for param in fragment.split('&') {
        let (key, val) = param.split_once('=')?;
        match key.trim() {
            "page" => {
                let p: u32 = val.trim().parse().ok()?;
                page = Some(p);
            }
            "bbox" => {
                let parts: Vec<&str> = val.split(',').map(str::trim).collect();
                if parts.len() != 4 {
                    return None;
                }
                let x: f32 = parts[0].parse().ok()?;
                let y: f32 = parts[1].parse().ok()?;
                let w: f32 = parts[2].parse().ok()?;
                let h: f32 = parts[3].parse().ok()?;
                if !x.is_finite() || !y.is_finite() || !w.is_finite() || !h.is_finite() {
                    return None;
                }
                if w < 0.0 || h < 0.0 {
                    return None;
                }
                bbox = Some((x, y, w, h));
            }
            _ => {}
        }
    }

    let page = page?;
    let (x, y, w, h) = bbox?;

    Some(PdfCropAnchor {
        path: path.to_string(),
        page,
        x,
        y,
        w,
        h,
    })
}

/// Computes a rectangular bounding box from drag start and current positions clamped to page bounds.
///
/// Ensures minimum/maximum ordering and returns `(x, y, width, height)`.
pub fn compute_crop_rect(
    start_pos: (f32, f32),
    current_pos: (f32, f32),
    page_bounds: (f32, f32),
) -> (f32, f32, f32, f32) {
    let bound_w = if page_bounds.0.is_finite() && page_bounds.0 > 0.0 {
        page_bounds.0
    } else {
        0.0
    };
    let bound_h = if page_bounds.1.is_finite() && page_bounds.1 > 0.0 {
        page_bounds.1
    } else {
        0.0
    };

    let sx = if start_pos.0.is_finite() {
        start_pos.0.clamp(0.0, bound_w)
    } else {
        0.0
    };
    let sy = if start_pos.1.is_finite() {
        start_pos.1.clamp(0.0, bound_h)
    } else {
        0.0
    };
    let cx = if current_pos.0.is_finite() {
        current_pos.0.clamp(0.0, bound_w)
    } else {
        0.0
    };
    let cy = if current_pos.1.is_finite() {
        current_pos.1.clamp(0.0, bound_h)
    } else {
        0.0
    };

    let x = sx.min(cx);
    let y = sy.min(cy);
    let w = sx.max(cx) - x;
    let h = sy.max(cy) - y;

    (x, y, w, h)
}

/// Checks if a bounding box has finite, non-negative coordinates and non-zero positive area.
pub fn is_valid_crop_rect(x: f32, y: f32, w: f32, h: f32) -> bool {
    x.is_finite()
        && y.is_finite()
        && w.is_finite()
        && h.is_finite()
        && x >= 0.0
        && y >= 0.0
        && w > 0.0
        && h > 0.0
}

/// Checks if a `PdfCropAnchor` points to a non-empty path, valid page (>= 1), and valid non-zero crop area.
pub fn is_valid_crop_anchor(anchor: &PdfCropAnchor) -> bool {
    !anchor.path.trim().is_empty()
        && anchor.page >= 1
        && is_valid_crop_rect(anchor.x, anchor.y, anchor.w, anchor.h)
}

/// Calculates the area of a bounding box. Returns 0.0 if any dimension is non-positive or non-finite.
pub fn crop_rect_area(w: f32, h: f32) -> f32 {
    if w.is_finite() && h.is_finite() && w > 0.0 && h > 0.0 {
        w * h
    } else {
        0.0
    }
}

/// Normalizes point-based coordinates into [0.0, 1.0] relative to `page_bounds`.
pub fn points_to_normalized(
    rect: (f32, f32, f32, f32),
    page_bounds: (f32, f32),
) -> (f32, f32, f32, f32) {
    if page_bounds.0 <= 0.0 || page_bounds.1 <= 0.0 {
        return (0.0, 0.0, 0.0, 0.0);
    }
    (
        rect.0 / page_bounds.0,
        rect.1 / page_bounds.1,
        rect.2 / page_bounds.0,
        rect.3 / page_bounds.1,
    )
}

/// Converts normalized [0.0, 1.0] coordinates into point coordinates given `page_bounds`.
pub fn normalized_to_points(
    rect: (f32, f32, f32, f32),
    page_bounds: (f32, f32),
) -> (f32, f32, f32, f32) {
    let pw = page_bounds.0.max(0.0);
    let ph = page_bounds.1.max(0.0);
    (rect.0 * pw, rect.1 * ph, rect.2 * pw, rect.3 * ph)
}
