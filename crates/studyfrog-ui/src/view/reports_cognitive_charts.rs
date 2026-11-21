//! Painter charts of the cognitive profile: the latency scatter and the weekly recall line.

use eframe::egui::{self, Align2, Pos2, Rect, Sense, Shape, Stroke, Vec2};

use crate::theme::palette_of;

/// Height of a chart, in points.
const HEIGHT: f32 = 220.0;
/// Room left of the plot for the value labels, in points.
const LEFT: f32 = 64.0;
/// Room below the plot for the value labels, in points.
const BOTTOM: f32 = 30.0;
/// Widest a chart is drawn, in points.
const MAX_WIDTH: f32 = 560.0;

/// The plotting area, with a frame allocated at the available width.
fn area(ui: &mut egui::Ui) -> (Rect, egui::Painter) {
    let width = ui.available_width().min(MAX_WIDTH);
    let (resp, painter) = ui.allocate_painter(Vec2::new(width, HEIGHT), Sense::hover());
    let r = resp.rect;
    let plot = Rect::from_min_max(
        Pos2::new(r.left() + LEFT, r.top() + 12.0),
        Pos2::new(r.right() - 12.0, r.bottom() - BOTTOM),
    );
    (plot, painter)
}

/// The two axes, drawn from the plotting area's bottom-left corner.
fn axes(painter: &egui::Painter, r: Rect, color: egui::Color32) {
    let stroke = Stroke::new(1.0, color);
    painter.line_segment([r.left_bottom(), r.right_bottom()], stroke);
    painter.line_segment([r.left_bottom(), r.left_top()], stroke);
}

fn label(painter: &egui::Painter, at: Pos2, align: Align2, text: &str, color: egui::Color32) {
    painter.text(
        at,
        align,
        text,
        crate::typo::painter_font(painter, crate::typo::Tier::Caption),
        color,
    );
}

/// Each answer as a point: the time the words should take against the time it took. Points under
/// the dashed diagonal were answered faster than expected.
pub fn latency_scatter(ui: &mut egui::Ui, points: &[(f32, f32)]) {
    let pal = palette_of(ui);
    let (r, painter) = area(ui);
    axes(&painter, r, pal.border);
    let top = points.iter().fold(1.0_f32, |m, (x, y)| m.max(*x).max(*y));
    let at = |x: f32, y: f32| {
        Pos2::new(
            r.left() + x / top * r.width(),
            r.bottom() - y / top * r.height(),
        )
    };
    painter.line_segment(
        [at(0.0, 0.0), at(top, top)],
        Stroke::new(1.0, pal.border.gamma_multiply(0.6)),
    );
    for (x, y) in points {
        painter.circle_filled(at(*x, *y), 2.5, pal.accent.gamma_multiply(0.7));
    }
    label(
        &painter,
        r.left_bottom() + Vec2::new(0.0, 4.0),
        Align2::LEFT_TOP,
        "0 s",
        pal.text_weak,
    );
    label(
        &painter,
        r.right_bottom() + Vec2::new(0.0, 4.0),
        Align2::RIGHT_TOP,
        &format!("{top:.0} s words"),
        pal.text_weak,
    );
    label(
        &painter,
        r.left_top() - Vec2::new(4.0, 0.0),
        Align2::RIGHT_TOP,
        &format!("{top:.0} s answer"),
        pal.text_weak,
    );
}

/// A line over weeks: `points` are (week, value) and the value runs from zero to `top`, which the
/// top label names (for example `100%` or `12 days`).
pub fn weekly_line(ui: &mut egui::Ui, points: &[(f32, f32)], top: f32, top_label: &str) {
    let pal = palette_of(ui);
    let (r, painter) = area(ui);
    axes(&painter, r, pal.border);
    let last = points.iter().fold(1.0_f32, |m, (w, _)| m.max(*w));
    let top = top.max(f32::EPSILON);
    let at = |w: f32, v: f32| {
        Pos2::new(
            r.left() + w / last * r.width(),
            r.bottom() - (v / top).clamp(0.0, 1.0) * r.height(),
        )
    };
    let pts: Vec<Pos2> = points.iter().map(|(w, v)| at(*w, *v)).collect();
    if pts.len() > 1 {
        painter.add(Shape::line(pts.clone(), Stroke::new(2.0, pal.accent)));
    }
    for p in pts {
        painter.circle_filled(p, 3.0, pal.accent);
    }
    label(
        &painter,
        r.left_top() - Vec2::new(4.0, 0.0),
        Align2::RIGHT_TOP,
        top_label,
        pal.text_weak,
    );
    label(
        &painter,
        r.left_bottom() - Vec2::new(4.0, 0.0),
        Align2::RIGHT_BOTTOM,
        "0",
        pal.text_weak,
    );
    label(
        &painter,
        r.right_bottom() + Vec2::new(0.0, 4.0),
        Align2::RIGHT_TOP,
        &format!("week {last:.0}"),
        pal.text_weak,
    );
}
