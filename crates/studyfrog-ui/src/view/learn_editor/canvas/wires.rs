//! The wires of the learn-mode canvas (STORY-0198): where each one runs, how it is drawn, and whether a click is on it.

use eframe::egui::{self, FontId, Pos2, Rect, Stroke, Vec2, vec2};

use super::NODE;

/// How close a click must be to a wire, in points, to select it.
const WIRE_TOUCH: f32 = 6.0;

/// The wire (by index) whose drawn path is within reach of the point, if any; the nearest one wins.
pub(super) fn wire_at(paths: &[(usize, Vec<Pos2>)], at: Pos2) -> Option<usize> {
    paths
        .iter()
        .filter_map(|(i, path)| distance_to_path(path, at).map(|d| (*i, d)))
        .filter(|(_, d)| *d <= WIRE_TOUCH)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

/// The distance from the point to the nearest segment of the path (`None` for a path with no segment).
pub fn distance_to_path(path: &[Pos2], at: Pos2) -> Option<f32> {
    path.windows(2)
        .map(|w| distance_to_segment(at, w[0], w[1]))
        .min_by(f32::total_cmp)
}

fn distance_to_segment(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let len2 = ab.length_sq();
    let t = if len2 <= f32::EPSILON {
        0.0
    } else {
        ((p - a).dot(ab) / len2).clamp(0.0, 1.0)
    };
    (a + ab * t - p).length()
}

/// Where the segment from `center` towards `toward` leaves a box of half-size `half`.
fn edge_point(center: Pos2, toward: Pos2, half: Vec2) -> Pos2 {
    let d = toward - center;
    if d.length() < 1.0 {
        return center;
    }
    let tx = if d.x.abs() > 1e-3 {
        half.x / d.x.abs()
    } else {
        f32::INFINITY
    };
    let ty = if d.y.abs() > 1e-3 {
        half.y / d.y.abs()
    } else {
        f32::INFINITY
    };
    center + d * tx.min(ty).min(1.0)
}

/// The points a wire is drawn through, from its start to its arrow's tip. A wire that goes back to a step on the left
/// runs as an arc above the boxes, so it does not run through the steps between them. Empty for a wire from a step
/// to itself.
pub fn wire_path(from: Rect, to: Rect) -> Vec<Pos2> {
    let half = NODE * 0.5;
    let (c1, c2) = (from.center(), to.center());
    if c1 == c2 {
        return Vec::new();
    }
    if c2.x < c1.x - 1.0 {
        let top = from.top().min(to.top()) - 28.0;
        return vec![
            Pos2::new(c1.x, from.top()),
            Pos2::new(c1.x, top),
            Pos2::new(c2.x, top),
            Pos2::new(c2.x, to.top()),
        ];
    }
    vec![edge_point(c1, c2, half), edge_point(c2, c1, half)]
}

/// Draw a wire along its path, with an arrow at its end and its condition written at its middle.
pub(super) fn wire(
    painter: &egui::Painter,
    path: &[Pos2],
    when: Option<&str>,
    stroke: Stroke,
    caption: f32,
) {
    let [.., prev, last] = path else { return };
    painter.add(egui::Shape::line(path.to_vec(), stroke));
    arrow_head(painter, *prev, *last, stroke);
    if let Some(w) = when {
        // The middle of the wire: the middle of its one segment, or the middle of the arc's top run.
        let (at, align) = if path.len() == 2 {
            (
                (path[0] + path[1].to_vec2()) * 0.5,
                egui::Align2::CENTER_CENTER,
            )
        } else {
            (
                Pos2::new((path[1].x + path[2].x) * 0.5, path[1].y - 2.0),
                egui::Align2::CENTER_BOTTOM,
            )
        };
        painter.text(at, align, w, FontId::proportional(caption), stroke.color);
    }
}

/// The two short strokes that make the head of an arrow pointing from `from` to `to`.
fn arrow_head(painter: &egui::Painter, from: Pos2, to: Pos2, stroke: Stroke) {
    let u = (to - from).normalized();
    let side = vec2(-u.y, u.x) * 5.0;
    painter.line_segment([to, to - u * 10.0 + side], stroke);
    painter.line_segment([to, to - u * 10.0 - side], stroke);
}
