//! Drawing an image occlusion: the picture scaled into the room there is, and the masks over it (STORY-0109).

use eframe::egui::{self, Color32, Rect, Stroke, vec2};
use studyfrog_md::occlusion::{Mask, OcclusionMode, unit_of};

use crate::theme::palette_of;

/// The screen rectangle of a fraction rectangle inside `frame`.
pub fn screen_rect(frame: Rect, (x, y, w, h): (f32, f32, f32, f32)) -> Rect {
    let min = frame.min + vec2(x * frame.width(), y * frame.height());
    Rect::from_min_size(min, vec2(w * frame.width(), h * frame.height()))
}

/// The picture at most `max` wide and 440 high, with the rectangle it takes; `None` (after saying why)
/// while it loads or when it cannot be shown.
pub fn picture(
    ui: &mut egui::Ui,
    path: &str,
    max: f32,
    sense: egui::Sense,
) -> Option<(Rect, egui::Response)> {
    let tex = match crate::view::image::vault_texture(ui, path) {
        Ok(Some(t)) => t,
        Ok(None) => {
            ui.weak(format!("[loading picture: {path}]"));
            return None;
        }
        Err(why) => {
            ui.weak(why);
            return None;
        }
    };
    let scale = (max / tex.size.x).min(440.0 / tex.size.y).min(1.5);
    let size = tex.size * scale;
    let (rect, response) = ui.allocate_exact_size(size, sense);
    egui::Image::from_texture(tex).paint_at(ui, rect);
    Some((rect, response))
}

/// One mask: opaque (hiding), marked (the one asked), or only outlined (shown).
pub fn paint_mask(ui: &egui::Ui, frame: Rect, m: &Mask, (hide, marked): (bool, bool)) {
    let p = palette_of(ui);
    let r = screen_rect(frame, unit_of(m));
    let fill = match (hide, marked) {
        (true, true) => p.accent,
        (true, false) => p.surface,
        (false, _) => Color32::TRANSPARENT,
    };
    ui.painter().rect_filled(r, 2.0, fill);
    let edge = if marked { p.warning } else { p.text_weak };
    ui.painter().rect_stroke(
        r,
        2.0,
        Stroke::new(if marked { 3.0 } else { 1.0 }, edge),
        egui::StrokeKind::Inside,
    );
}

/// The picture of a run: before the answer the asked mask is marked and hidden (with every other mask
/// in Hide all); after it, the asked mask is uncovered and outlined.
pub fn run_picture(
    ui: &mut egui::Ui,
    v: &studyfrog_service::content_occlusion::OcclusionView,
    revealed: bool,
) {
    let width = ui.available_width().min(720.0);
    let Some((frame, response)) = picture(ui, &v.image, width, egui::Sense::hover()) else {
        return;
    };
    for (i, m) in v.masks.iter().enumerate() {
        let asked = i == v.active;
        let hide = if asked {
            !revealed
        } else {
            v.mode == OcclusionMode::HideAll
        };
        paint_mask(ui, frame, m, (hide, asked));
    }
    let what = if revealed {
        "uncovered"
    } else {
        "hidden and marked"
    };
    let name = format!(
        "Picture with {} masks, mask {} {what}",
        v.masks.len(),
        v.active + 1
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Image, true, name.clone()));
}

/// The number and label of a mask, written inside it (the editor).
pub fn caption(ui: &egui::Ui, frame: Rect, m: &Mask, n: usize) {
    let r = screen_rect(frame, unit_of(m));
    let text = if m.label.is_empty() {
        format!("{n}")
    } else {
        format!("{n} {}", m.label)
    };
    let color = palette_of(ui).text;
    ui.painter().with_clip_rect(r).text(
        r.left_top() + vec2(4.0, 2.0),
        egui::Align2::LEFT_TOP,
        text,
        egui::FontId::proportional(13.0),
        color,
    );
}
