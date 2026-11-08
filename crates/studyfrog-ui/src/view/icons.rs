//! Icon buttons for the top bar: drawn with shapes or a symbol, named for assistive technology.

use eframe::egui::{
    self, Align2, Color32, FontId, Pos2, Rect, Response, Role, Sense, Stroke, WidgetInfo, vec2,
};

/// Which picture an icon button shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Glyph {
    /// A window with a side panel.
    Sidebar,
    /// A grid of four squares.
    Grid,
    /// A bell.
    Bell,
    /// A cog.
    Gear,
    /// A head and shoulders.
    Person,
    /// A question mark.
    Help,
    /// A cross.
    Close,
    /// An arrow pointing left.
    Back,
    /// An arrow pointing right.
    Forward,
    /// A clock face.
    History,
    /// A stopwatch.
    Timer,
    /// Four corners around an empty middle: Zen mode.
    Focus,
}

const SIZE: f32 = 32.0;

fn paint(ui: &egui::Ui, rect: Rect, glyph: Glyph, color: Color32) {
    let p = ui.painter();
    let c = rect.center();
    let stroke = Stroke::new(1.5, color);
    match glyph {
        Glyph::Sidebar => {
            let frame = Rect::from_center_size(c, vec2(16.0, 14.0));
            p.rect_stroke(frame, 2.0, stroke, egui::StrokeKind::Inside);
            p.line_segment(
                [
                    Pos2::new(frame.left() + 5.5, frame.top()),
                    Pos2::new(frame.left() + 5.5, frame.bottom()),
                ],
                stroke,
            );
        }
        Glyph::Grid => {
            for (dx, dy) in [(-4.5, -4.5), (4.5, -4.5), (-4.5, 4.5), (4.5, 4.5)] {
                let cell = Rect::from_center_size(Pos2::new(c.x + dx, c.y + dy), vec2(6.0, 6.0));
                p.rect_stroke(cell, 1.5, stroke, egui::StrokeKind::Inside);
            }
        }
        Glyph::Focus => corners(p, c, stroke),
        Glyph::Bell => bell(p, c, stroke),
        Glyph::Gear => gear(p, c, stroke),
        Glyph::Person => {
            p.circle_stroke(Pos2::new(c.x, c.y - 4.0), 4.0, stroke);
            let shoulders: Vec<Pos2> = arc(Pos2::new(c.x, c.y + 8.0), 8.0, 6.5, 180.0, 360.0);
            p.add(egui::Shape::line(shoulders, stroke));
        }
        Glyph::Back | Glyph::Forward => {
            let s = if glyph == Glyph::Back { -1.0 } else { 1.0 };
            let tip = Pos2::new(c.x + 6.0 * s, c.y);
            p.line_segment([Pos2::new(c.x - 6.0 * s, c.y), tip], stroke);
            p.line_segment([Pos2::new(c.x + 1.0 * s, c.y - 6.0), tip], stroke);
            p.line_segment([Pos2::new(c.x + 1.0 * s, c.y + 6.0), tip], stroke);
        }
        Glyph::History => {
            p.circle_stroke(c, 8.0, stroke);
            p.line_segment([c, Pos2::new(c.x, c.y - 5.0)], stroke);
            p.line_segment([c, Pos2::new(c.x + 4.0, c.y + 2.5)], stroke);
        }
        Glyph::Timer => {
            let m = Pos2::new(c.x, c.y + 1.5);
            p.circle_stroke(m, 7.0, stroke);
            p.line_segment(
                [
                    Pos2::new(c.x - 2.5, c.y - 8.5),
                    Pos2::new(c.x + 2.5, c.y - 8.5),
                ],
                stroke,
            );
            p.line_segment([m, Pos2::new(m.x + 3.0, m.y - 4.0)], stroke);
        }
        Glyph::Close => {
            let d = 5.0;
            p.line_segment(
                [Pos2::new(c.x - d, c.y - d), Pos2::new(c.x + d, c.y + d)],
                stroke,
            );
            p.line_segment(
                [Pos2::new(c.x - d, c.y + d), Pos2::new(c.x + d, c.y - d)],
                stroke,
            );
        }
        Glyph::Help => {
            p.text(
                c,
                Align2::CENTER_CENTER,
                "?",
                FontId::proportional(17.0),
                color,
            );
        }
    }
}

/// Points on an ellipse arc, angles in degrees (0 = right, 90 = down).
fn arc(center: Pos2, rx: f32, ry: f32, from: f32, to: f32) -> Vec<Pos2> {
    (0..=16)
        .map(|i| {
            let a = (from + (to - from) * i as f32 / 16.0).to_radians();
            Pos2::new(center.x + rx * a.cos(), center.y + ry * a.sin())
        })
        .collect()
}

fn corners(p: &egui::Painter, c: Pos2, stroke: Stroke) {
    for (sx, sy) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        let corner = Pos2::new(c.x + 7.0 * sx, c.y + 6.0 * sy);
        let along = Pos2::new(corner.x - 4.0 * sx, corner.y);
        let down = Pos2::new(corner.x, corner.y - 4.0 * sy);
        p.add(egui::Shape::line(vec![along, corner, down], stroke));
    }
}

fn bell(p: &egui::Painter, c: Pos2, stroke: Stroke) {
    let mut outline = vec![
        Pos2::new(c.x - 8.0, c.y + 5.0),
        Pos2::new(c.x - 6.0, c.y + 3.0),
    ];
    outline.extend(arc(Pos2::new(c.x, c.y - 1.0), 6.0, 7.0, 180.0, 360.0));
    outline.extend([
        Pos2::new(c.x + 6.0, c.y + 3.0),
        Pos2::new(c.x + 8.0, c.y + 5.0),
        Pos2::new(c.x - 8.0, c.y + 5.0),
    ]);
    p.add(egui::Shape::line(outline, stroke));
    p.circle_stroke(Pos2::new(c.x, c.y + 8.0), 1.8, stroke);
}

fn gear(p: &egui::Painter, c: Pos2, stroke: Stroke) {
    p.circle_stroke(c, 5.0, stroke);
    for k in 0..8 {
        let a = (k as f32 * 45.0).to_radians();
        let (dx, dy) = (a.cos(), a.sin());
        p.line_segment(
            [
                Pos2::new(c.x + 6.0 * dx, c.y + 6.0 * dy),
                Pos2::new(c.x + 8.5 * dx, c.y + 8.5 * dy),
            ],
            Stroke::new(2.5, stroke.color),
        );
    }
    p.circle_filled(c, 1.8, stroke.color);
}

/// A square icon button. `active` keeps it highlighted (its menu is open). The name is what a
/// screen reader announces.
pub fn icon_button(ui: &mut egui::Ui, glyph: Glyph, name: &str, active: bool) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(SIZE, SIZE), Sense::click());
    let visuals = ui.style().interact(&response);
    if active || response.hovered() || response.has_focus() {
        let fill = if active {
            ui.visuals().widgets.active.weak_bg_fill
        } else {
            visuals.weak_bg_fill
        };
        ui.painter().rect_filled(rect, 6.0, fill);
    }
    paint(ui, rect, glyph, visuals.text_color());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, name));
    response.clone().on_hover_text(name)
}
