//! Painter charts of the cognitive profile: the latency scatter, curves over time and the answer-time
//! distribution.

use eframe::egui::{self, Align2, Pos2, Rect, Sense, Shape, Stroke, Vec2};

use crate::theme::palette_of;

/// Height of a chart, in points.
const HEIGHT: f32 = 220.0;
/// Room left of the plot for the value labels, in points.
const LEFT: f32 = 88.0;
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

/// A curve over x (from zero to the last x) and y (from zero to `top`, which `top_label` names).
/// `end_label` is written under the last x, for example `week 3`.
pub fn curve(ui: &mut egui::Ui, points: &[(f32, f32)], top: f32, top_label: &str, end_label: &str) {
    let pal = palette_of(ui);
    let (r, painter) = area(ui);
    axes(&painter, r, pal.border);
    let last = points.iter().fold(1.0_f32, |m, (x, _)| m.max(*x));
    let top = top.max(f32::EPSILON);
    let at = |x: f32, y: f32| {
        Pos2::new(
            r.left() + x / last * r.width(),
            r.bottom() - (y / top).clamp(0.0, 1.0) * r.height(),
        )
    };
    let pts: Vec<Pos2> = points.iter().map(|(x, y)| at(*x, *y)).collect();
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
        end_label,
        pal.text_weak,
    );
}

/// Counts per bin as bars from zero. Bin `i` starts at `i * bin_s` seconds; the last bin holds
/// everything longer, and its label ends in `+`.
pub fn bars(ui: &mut egui::Ui, counts: &[u32], bin_s: u32) {
    let pal = palette_of(ui);
    let (r, painter) = area(ui);
    axes(&painter, r, pal.border);
    let top = counts.iter().copied().max().unwrap_or(0).max(1) as f32;
    let slot = r.width() / counts.len().max(1) as f32;
    for (i, count) in counts.iter().enumerate() {
        let x0 = r.left() + i as f32 * slot + slot * 0.15;
        let x1 = r.left() + (i as f32 + 1.0) * slot - slot * 0.15;
        let h = *count as f32 / top * r.height();
        let bar = Rect::from_min_max(Pos2::new(x0, r.bottom() - h), Pos2::new(x1, r.bottom()));
        painter.rect_filled(bar, 2.0, pal.accent.gamma_multiply(0.7));
        let start = i as u32 * bin_s;
        let text = if i + 1 == counts.len() {
            format!("{start}+ s")
        } else {
            format!("{start}")
        };
        label(
            &painter,
            Pos2::new((x0 + x1) / 2.0, r.bottom() + 4.0),
            Align2::CENTER_TOP,
            &text,
            pal.text_weak,
        );
    }
    label(
        &painter,
        r.left_top() - Vec2::new(4.0, 0.0),
        Align2::RIGHT_TOP,
        &format!("{top:.0} answers"),
        pal.text_weak,
    );
}
