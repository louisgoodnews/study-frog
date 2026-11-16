//! Directed Bézier edge rendering, anchor minimization, arrowheads, and labels.
//!
//! Follows Invariant P1: zero inherent impl blocks; logic in free functions.

use eframe::egui::{Color32, Painter, Pos2, Rect, Stroke, Vec2, epaint::CubicBezierShape};
use studyfrog_graph::canvas::{CanvasEdge, CanvasEdgeDir};

/// Anchor side on a node's bounding rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnchorSide {
    /// Top center edge.
    Top,
    /// Bottom center edge.
    Bottom,
    /// Left center edge.
    Left,
    /// Right center edge.
    Right,
}

/// Normal outward direction vector for an anchor side.
pub fn anchor_normal(side: AnchorSide) -> Vec2 {
    match side {
        AnchorSide::Top => Vec2::new(0.0, -1.0),
        AnchorSide::Bottom => Vec2::new(0.0, 1.0),
        AnchorSide::Left => Vec2::new(-1.0, 0.0),
        AnchorSide::Right => Vec2::new(1.0, 0.0),
    }
}

/// Computes the 4 perimeter anchor points of a node rectangle.
pub fn perimeter_anchors(r: Rect) -> [(Pos2, AnchorSide); 4] {
    [
        (Pos2::new(r.center().x, r.min.y), AnchorSide::Top),
        (Pos2::new(r.center().x, r.max.y), AnchorSide::Bottom),
        (Pos2::new(r.min.x, r.center().y), AnchorSide::Left),
        (Pos2::new(r.max.x, r.center().y), AnchorSide::Right),
    ]
}

/// Selects the pair of anchors between two rectangles that minimizes Euclidean distance.
pub fn best_anchors(from: Rect, to: Rect) -> (Pos2, AnchorSide, Pos2, AnchorSide) {
    let from_anchors = perimeter_anchors(from);
    let to_anchors = perimeter_anchors(to);
    let mut best_dist = f32::MAX;
    let mut best = (
        from_anchors[0].0,
        from_anchors[0].1,
        to_anchors[0].0,
        to_anchors[0].1,
    );

    for &(p1, s1) in &from_anchors {
        for &(p2, s2) in &to_anchors {
            let d = p1.distance(p2);
            if d < best_dist {
                best_dist = d;
                best = (p1, s1, p2, s2);
            }
        }
    }
    best
}

/// Evaluates a cubic Bézier curve at parameter $t \in [0, 1]$.
pub fn eval_cubic_bezier(p0: Pos2, c1: Pos2, c2: Pos2, p1: Pos2, t: f32) -> Pos2 {
    let u = 1.0 - t;
    let tt = t * t;
    let uu = u * u;
    let uuu = uu * u;
    let ttt = tt * t;

    let x = uuu * p0.x + 3.0 * uu * t * c1.x + 3.0 * u * tt * c2.x + ttt * p1.x;
    let y = uuu * p0.y + 3.0 * uu * t * c1.y + 3.0 * u * tt * c2.y + ttt * p1.y;
    Pos2::new(x, y)
}

/// Draws an arrowhead triangle pointing in the direction of `tangent` at `tip`.
pub fn draw_arrowhead(painter: &Painter, tip: Pos2, tangent: Vec2, color: Color32) {
    let len = tangent.length();
    let u = if len > 1e-4 {
        tangent / len
    } else {
        Vec2::new(1.0, 0.0)
    };
    let normal = Vec2::new(-u.y, u.x);
    let size = 9.0;
    let base = tip - u * size;
    let w1 = base + normal * (size * 0.45);
    let w2 = base - normal * (size * 0.45);
    painter.add(eframe::egui::epaint::PathShape::convex_polygon(
        vec![tip, w1, w2],
        color,
        Stroke::NONE,
    ));
}

/// Renders a midpoint relationship label pill on the painter.
pub fn draw_label_pill(painter: &Painter, mid: Pos2, label: &str, bg: Color32, fg: Color32) {
    let text = if label.chars().count() > 30 {
        let truncated: String = label.chars().take(28).collect();
        format!("{truncated}…")
    } else {
        label.to_string()
    };
    let font_id = crate::typo::painter_font(painter, crate::typo::Tier::Caption);
    let text_shape = painter.layout_no_wrap(text, font_id, fg);
    let pad = Vec2::new(6.0, 3.0);
    let rect = Rect::from_center_size(mid, text_shape.size() + pad * 2.0);

    painter.rect_filled(rect, 4.0, bg);
    painter.rect_stroke(
        rect,
        4.0,
        Stroke::new(1.0, fg.linear_multiply(0.3)),
        eframe::egui::StrokeKind::Inside,
    );
    let text_pos = Pos2::new(rect.min.x + pad.x, rect.min.y + pad.y);
    painter.galley(text_pos, text_shape, fg);
}

/// Draws a directed Bézier edge between source and target node rectangles.
pub fn draw_edge(
    painter: &Painter,
    edge: &CanvasEdge,
    from_rect: Rect,
    to_rect: Rect,
    color: Color32,
    bg_color: Color32,
) {
    let stroke = Stroke::new(1.5, color);
    let is_self_loop = from_rect == to_rect;

    let (p0, c1, c2, p1) = if is_self_loop {
        let w = from_rect.width();
        let top_y = from_rect.min.y;
        let pt0 = Pos2::new(from_rect.min.x + w * 0.3, top_y);
        let pt1 = Pos2::new(from_rect.min.x + w * 0.7, top_y);
        let ctrl1 = Pos2::new(from_rect.min.x + w * 0.1, top_y - 45.0);
        let ctrl2 = Pos2::new(from_rect.min.x + w * 0.9, top_y - 45.0);
        (pt0, ctrl1, ctrl2, pt1)
    } else {
        let (pt0, s0, pt1, s1) = best_anchors(from_rect, to_rect);
        let dist = pt0.distance(pt1);
        let delta = (dist * 0.35).clamp(20.0, 120.0);
        let ctrl1 = pt0 + anchor_normal(s0) * delta;
        let ctrl2 = pt1 + anchor_normal(s1) * delta;
        (pt0, ctrl1, ctrl2, pt1)
    };

    painter.add(CubicBezierShape::from_points_stroke(
        [p0, c1, c2, p1],
        false,
        Color32::TRANSPARENT,
        stroke,
    ));

    match edge.direction {
        CanvasEdgeDir::Forward => {
            draw_arrowhead(painter, p1, p1 - c2, color);
        }
        CanvasEdgeDir::Backward => {
            draw_arrowhead(painter, p0, p0 - c1, color);
        }
        CanvasEdgeDir::Bi => {
            draw_arrowhead(painter, p1, p1 - c2, color);
            draw_arrowhead(painter, p0, p0 - c1, color);
        }
        CanvasEdgeDir::Undirected => {}
    }

    if let Some(label) = &edge.label {
        let mid = eval_cubic_bezier(p0, c1, c2, p1, 0.5);
        draw_label_pill(painter, mid, label, bg_color, color);
    }
}
