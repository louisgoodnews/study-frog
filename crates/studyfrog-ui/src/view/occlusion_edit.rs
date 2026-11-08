//! The image occlusion editor on a card's page (STORY-0109): drag on the picture to draw a mask, drag a
//! mask to move it, drag its corner to resize it, click it to select it and give it a label or delete it.
//! Each finished gesture is one edit of the card, saved like any other.

use eframe::egui::{self, Id, Pos2, RichText};
use studyfrog_md::doc::Doc;
use studyfrog_md::occlusion::{Occlusion, OcclusionMode, images_in};

use crate::editor::Editor;
use crate::msg::{EditAction, Msg};
use crate::occlusion_ui::{Gesture, finish, preview, relabel, remove, start};
use crate::view::occlusion_paint::{paint_mask, picture, screen_rect};

fn set(out: &mut Vec<Msg>, o: Option<Occlusion>) {
    out.push(Msg::Edit(EditAction::Occlusion(o)));
}

fn unit(frame: egui::Rect, p: Pos2) -> (f32, f32) {
    (
        (p.x - frame.min.x) / frame.width(),
        (p.y - frame.min.y) / frame.height(),
    )
}

/// The section, for a card whose front shows a picture or that has masks already.
pub fn section(e: &Editor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if !matches!(e.doc, Doc::Card(_)) {
        return;
    }
    let front = e.form.fields.first().map_or("", |f| f.text.as_str());
    let Some(o) = &e.form.occlusion else {
        if let Some(image) = images_in(front).into_iter().next() {
            let make = "Make this an image occlusion card";
            if ui
                .button(make)
                .on_hover_text("Hide parts of the picture and ask for them one by one")
                .clicked()
            {
                set(
                    out,
                    Some(Occlusion {
                        image,
                        mode: OcclusionMode::HideAll,
                        masks: Vec::new(),
                    }),
                );
            }
        }
        return;
    };
    ui.label(RichText::new("Image occlusion").strong());
    ui.weak("Drag on the picture to hide a part; drag a box to move it, its lower right corner to resize it.");
    ui.horizontal_wrapped(|ui| {
        for (mode, name) in [
            (OcclusionMode::HideAll, "Hide all, guess one"),
            (OcclusionMode::HideOne, "Hide one, guess one"),
        ] {
            if ui.radio(o.mode == mode, name).clicked() && o.mode != mode {
                set(out, Some(Occlusion { mode, ..o.clone() }));
            }
        }
        if ui.button("Stop using occlusion").clicked() {
            set(out, None);
        }
    });
    let id = ui.make_persistent_id(("occlusion", e.uid.0));
    canvas(o, id, ui, out);
    selected_row(o, id, ui, out);
}

fn canvas(o: &Occlusion, id: Id, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let width = ui.available_width().min(720.0);
    let Some((frame, r)) = picture(ui, &o.image, width, egui::Sense::click_and_drag()) else {
        return;
    };
    let selected = ui.data(|d| d.get_temp::<usize>(id.with("selected")));
    let gesture_id = id.with("gesture");
    let pointer = r.interact_pointer_pos().map(|p| unit(frame, p));
    let origin = ui
        .input(|i| i.pointer.press_origin())
        .map(|p| unit(frame, p));
    if let (true, Some(p)) = (r.drag_started(), origin.or(pointer)) {
        ui.data_mut(|d| d.insert_temp(gesture_id, start(&o.masks, selected, p)));
    }
    let gesture = ui.data(|d| d.get_temp::<Gesture>(gesture_id));
    for (i, m) in o.masks.iter().enumerate() {
        let moving = matches!(gesture, Some(Gesture::Move(j, ..) | Gesture::Resize(j)) if j == i);
        if !moving {
            paint_mask(ui, frame, m, (true, selected == Some(i)));
            crate::view::occlusion_paint::caption(ui, frame, m, i + 1);
        }
    }
    if let (Some(g), Some(p)) = (gesture, pointer.or(r.hover_pos().map(|p| unit(frame, p)))) {
        if let Some(rect) = preview(&o.masks, g, p) {
            let s = screen_rect(frame, rect);
            ui.painter().rect_stroke(
                s,
                2.0,
                egui::Stroke::new(2.0, crate::theme::palette_of(ui).warning),
                egui::StrokeKind::Inside,
            );
        }
        if r.drag_stopped() {
            ui.data_mut(|d| d.remove::<Gesture>(gesture_id));
            if let Some((next, i)) = finish(o, g, p) {
                ui.data_mut(|d| d.insert_temp(id.with("selected"), i));
                set(out, Some(next));
            }
        }
    }
    if let (true, Some(p)) = (r.clicked(), pointer) {
        let hit = crate::occlusion_ui::hit(&o.masks, p);
        ui.data_mut(|d| match hit {
            Some(i) => {
                d.insert_temp(id.with("selected"), i);
            }
            None => d.remove::<usize>(id.with("selected")),
        });
    }
    let name = format!("Picture with {} masks: drag to draw a mask", o.masks.len());
    r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Image, true, name.clone()));
}

fn selected_row(o: &Occlusion, id: Id, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.weak(format!(
        "{} mask(s). Click one to label or delete it.",
        o.masks.len()
    ));
    let Some(i) = ui
        .data(|d| d.get_temp::<usize>(id.with("selected")))
        .filter(|i| *i < o.masks.len())
    else {
        return;
    };
    ui.horizontal(|ui| {
        let label = ui.label(format!("Label of mask {}", i + 1));
        let mut text = o.masks[i].label.clone();
        let edit = egui::TextEdit::singleline(&mut text).hint_text("what it hides");
        if ui.add(edit).labelled_by(label.id).changed() {
            set(out, Some(relabel(o, i, &text)));
        }
        if ui.button(format!("Delete mask {}", i + 1)).clicked() {
            ui.data_mut(|d| d.remove::<usize>(id.with("selected")));
            set(out, Some(remove(o, i)));
        }
    });
}
