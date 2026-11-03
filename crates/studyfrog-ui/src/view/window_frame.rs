//! StudyFrog's own window frame: the top bar is the title bar (drag it to move the window, double-click it to
//! maximise), with large minimise, maximise and red close buttons, and the window's edges resize it. Moving and
//! resizing are handed to the window manager, so snapping and tiling work as they do for any window.

use eframe::egui::{
    self, Color32, CursorIcon, Pos2, Rect, ResizeDirection, Role, Sense, Stroke, ViewportCommand,
    WidgetInfo, vec2,
};

use crate::chrome::ChromeMsg;
use crate::msg::Msg;
use crate::quit_ui::QuitMsg;
use crate::theme::palette_of;

/// Width of the band at the window's edge that resizes it.
const EDGE: f32 = 6.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Button {
    Minimize,
    Maximize,
    Restore,
    Close,
}

fn maximized(ctx: &egui::Context) -> bool {
    ctx.input(|i| i.viewport().maximized.unwrap_or(false))
}

fn paint(ui: &egui::Ui, rect: Rect, kind: Button, color: Color32) {
    let (c, s) = (rect.center(), 6.0);
    let stroke = Stroke::new(1.6, color);
    let p = ui.painter();
    match kind {
        Button::Minimize => {
            p.line_segment(
                [Pos2::new(c.x - s, c.y + 3.0), Pos2::new(c.x + s, c.y + 3.0)],
                stroke,
            );
        }
        Button::Maximize => {
            p.rect_stroke(
                Rect::from_center_size(c, vec2(2.0 * s, 2.0 * s)),
                2.0,
                stroke,
                egui::StrokeKind::Middle,
            );
        }
        Button::Restore => {
            let front =
                Rect::from_center_size(c + vec2(-1.5, 1.5), vec2(2.0 * s - 3.0, 2.0 * s - 3.0));
            let back = front.translate(vec2(3.5, -3.5));
            p.rect_stroke(front, 2.0, stroke, egui::StrokeKind::Middle);
            p.line_segment([back.left_top(), back.right_top()], stroke);
            p.line_segment([back.right_top(), back.right_bottom()], stroke);
        }
        Button::Close => {
            p.line_segment([c + vec2(-s, -s), c + vec2(s, s)], stroke);
            p.line_segment([c + vec2(-s, s), c + vec2(s, -s)], stroke);
        }
    }
}

/// One window button: a wide target with a hover fill (red for close) and the symbol painted on it.
fn button(ui: &mut egui::Ui, kind: Button, name: &str) -> egui::Response {
    let (rect, r) = ui.allocate_exact_size(vec2(44.0, 32.0), Sense::click());
    let p = palette_of(ui);
    let red = Color32::from_rgb(200, 48, 48);
    let (fill, symbol) = match (kind == Button::Close, r.hovered()) {
        (true, false) => (red, Color32::WHITE),
        (true, true) => (Color32::from_rgb(224, 66, 66), Color32::WHITE),
        (false, true) => (p.surface_hover, p.text),
        (false, false) => (Color32::TRANSPARENT, p.text),
    };
    ui.painter().rect_filled(rect, 6.0, fill);
    paint(ui, rect, kind, symbol);
    r.widget_info(|| WidgetInfo::labeled(Role::Button, true, name));
    r.on_hover_text(name)
}

/// The three buttons, right to left: close (red), maximise or restore, minimise.
pub fn controls(ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let ctx = ui.ctx().clone();
    if button(ui, Button::Close, "Exit StudyFrog").clicked() {
        out.push(Msg::Chrome(ChromeMsg::Quit(QuitMsg::Ask)));
    }
    ui.add_space(6.0); // a gap, so that a click aimed at maximise cannot land on close
    let full = maximized(&ctx);
    let (kind, name) = if full {
        (Button::Restore, "Restore window size")
    } else {
        (Button::Maximize, "Maximize window")
    };
    if button(ui, kind, name).clicked() {
        ctx.send_viewport_cmd(ViewportCommand::Maximized(!full));
    }
    if button(ui, Button::Minimize, "Minimize window").clicked() {
        ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
    }
}

/// Make the bar behind its buttons drag the window, and a double-click maximise or restore it. Call it first
/// in the bar, before the buttons, so that they stay on top of it.
pub fn drag_area(ui: &mut egui::Ui, rect: Rect) {
    let r = ui.interact(
        rect,
        ui.make_persistent_id("titlebar-drag"),
        Sense::click_and_drag(),
    );
    let ctx = ui.ctx();
    if r.double_clicked() {
        ctx.send_viewport_cmd(ViewportCommand::Maximized(!maximized(ctx)));
    } else if r.drag_started_by(egui::PointerButton::Primary) {
        ctx.send_viewport_cmd(ViewportCommand::StartDrag);
    }
}

fn direction_at(pos: Pos2, screen: Rect) -> Option<ResizeDirection> {
    let (l, r) = (
        pos.x - screen.left() <= EDGE,
        screen.right() - pos.x <= EDGE,
    );
    let (t, b) = (
        pos.y - screen.top() <= EDGE,
        screen.bottom() - pos.y <= EDGE,
    );
    Some(match (l, r, t, b) {
        (true, _, true, _) => ResizeDirection::NorthWest,
        (_, true, true, _) => ResizeDirection::NorthEast,
        (true, _, _, true) => ResizeDirection::SouthWest,
        (_, true, _, true) => ResizeDirection::SouthEast,
        (true, ..) => ResizeDirection::West,
        (_, true, ..) => ResizeDirection::East,
        (_, _, true, _) => ResizeDirection::North,
        (.., true) => ResizeDirection::South,
        _ => return None,
    })
}

fn cursor(d: ResizeDirection) -> CursorIcon {
    match d {
        ResizeDirection::North => CursorIcon::ResizeNorth,
        ResizeDirection::South => CursorIcon::ResizeSouth,
        ResizeDirection::East => CursorIcon::ResizeEast,
        ResizeDirection::West => CursorIcon::ResizeWest,
        ResizeDirection::NorthEast => CursorIcon::ResizeNorthEast,
        ResizeDirection::SouthEast => CursorIcon::ResizeSouthEast,
        ResizeDirection::NorthWest => CursorIcon::ResizeNorthWest,
        ResizeDirection::SouthWest => CursorIcon::ResizeSouthWest,
    }
}

/// The thin outline of the window and the resizing at its edges (not while maximised).
pub fn frame(ui: &egui::Ui) {
    let ctx = ui.ctx();
    if maximized(ctx) {
        return;
    }
    let screen = ctx.content_rect();
    let border = palette_of(ui).border;
    ui.painter().rect_stroke(
        screen,
        0.0,
        Stroke::new(1.0, border),
        egui::StrokeKind::Inside,
    );
    let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) else {
        return;
    };
    let Some(direction) = direction_at(pos, screen) else {
        return;
    };
    ctx.set_cursor_icon(cursor(direction));
    if ctx.input(|i| i.pointer.primary_pressed()) {
        ctx.send_viewport_cmd(ViewportCommand::BeginResize(direction));
    }
}
