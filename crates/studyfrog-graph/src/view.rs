//! Mapping layout positions to screen positions, into caller-owned buffers.

use crate::layout::Layout;

/// Pan and zoom of the graph canvas.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct View {
    /// Screen pixels per layout unit.
    pub zoom: f32,
    /// Layout x shown at the canvas centre.
    pub center_x: f32,
    /// Layout y shown at the canvas centre.
    pub center_y: f32,
}

/// Fit-to-canvas starting view.
pub fn view_fit(l: &Layout, canvas_width: f32, canvas_height: f32) -> View {
    let zoom = (canvas_width.min(canvas_height) / (2.0 * l.extent)).max(0.01);
    View {
        zoom,
        center_x: 0.0,
        center_y: 0.0,
    }
}

/// Write the screen position of every node into `sx`/`sy` (which must be as long as the layout).
/// `origin_*` is the canvas centre in screen pixels. Allocates nothing.
pub fn project(l: &Layout, v: &View, origin_x: f32, origin_y: f32, sx: &mut [f32], sy: &mut [f32]) {
    for i in 0..l.x.len().min(sx.len()).min(sy.len()) {
        sx[i] = origin_x + (l.x[i] - v.center_x) * v.zoom;
        sy[i] = origin_y + (l.y[i] - v.center_y) * v.zoom;
    }
}

/// The node nearest to the screen point within `radius` pixels, if any.
pub fn nearest(sx: &[f32], sy: &[f32], px: f32, py: f32, radius: f32) -> Option<usize> {
    let mut best: Option<(usize, f32)> = None;
    for i in 0..sx.len().min(sy.len()) {
        let d2 = (sx[i] - px).powi(2) + (sy[i] - py).powi(2);
        if d2 <= radius * radius && best.is_none_or(|(_, b)| d2 < b) {
            best = Some((i, d2));
        }
    }
    best.map(|(i, _)| i)
}
