//! Highlights and notes in the PDF screen: the colour and tool bar, and the list of every mark.

use eframe::egui::{self, Color32, RichText, Sense, TextEdit, Vec2};
use studyfrog_config::pdf_marks::{MARK_COLORS, MarkKind, PdfMark, mark_rgb};

use crate::msg::Msg;
use crate::pdf_mark_types::MarkMsg;
use crate::pdf_types::{PdfMsg, PdfView};

fn send(out: &mut Vec<Msg>, m: MarkMsg) {
    out.push(Msg::Pdf(PdfMsg::Marks(m)));
}

/// The colour of a mark as an egui colour.
pub fn color_of(name: &str) -> Color32 {
    let [r, g, b] = mark_rgb(name);
    Color32::from_rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

fn swatch(ui: &mut egui::Ui, name: &str, selected: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(22.0), Sense::click());
    let stroke = if selected {
        ui.visuals().selection.stroke
    } else {
        ui.visuals().widgets.inactive.bg_stroke
    };
    ui.painter()
        .rect_filled(rect.shrink(3.0), 4.0, color_of(name));
    ui.painter()
        .rect_stroke(rect.shrink(1.0), 5.0, stroke, egui::StrokeKind::Inside);
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::Role::Button, true, selected, format!("Colour {name}"))
    });
    response.on_hover_text(name)
}

/// The row under the toolbar: colour, highlight the selection, place a note.
pub fn bar(view: &PdfView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal_wrapped(|ui| {
        ui.label("Mark with");
        for (name, _) in MARK_COLORS {
            if swatch(ui, name, view.color == name).clicked() {
                send(out, MarkMsg::Color(name.to_owned()));
            }
        }
        let can = !view.selection.is_empty();
        if ui
            .add_enabled(can, egui::Button::new("Highlight selection"))
            .clicked()
        {
            send(out, MarkMsg::Highlight);
        }
        let place = egui::Button::selectable(view.placing_note, "Place note");
        if ui
            .add(place)
            .on_hover_text("Then click the page where the note belongs")
            .clicked()
        {
            send(out, MarkMsg::ToggleNoteMode);
        }
    });
}

fn excerpt(text: &str) -> String {
    let short: String = text.chars().take(120).collect();
    if text.chars().count() > 120 {
        format!("{short} ...")
    } else {
        short
    }
}

fn entry(view: &PdfView, at: usize, mark: &PdfMark, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.push_id(mark.id, |ui| {
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(14.0), Sense::hover());
            ui.painter().rect_filled(rect, 3.0, color_of(&mark.color));
            let kind = if mark.kind == MarkKind::Note {
                "Note"
            } else {
                "Highlight"
            };
            if ui.link(format!("{kind}, page {}", mark.page + 1)).clicked() {
                send(out, MarkMsg::Jump(mark.id));
            }
            if ui.small_button("Delete").clicked() {
                send(out, MarkMsg::Delete(mark.id));
            }
        });
        match view.marks_now.get(at).copied().flatten() {
            Some(p) if p != mark.page => {
                ui.weak(format!("The text is now on page {}.", p + 1));
            }
            None if !mark.covered.is_empty() => {
                ui.weak("The text is no longer in this PDF.");
            }
            _ => {}
        }
        if !mark.covered.is_empty() {
            ui.label(RichText::new(excerpt(&mark.covered)).italics());
        }
        let mut note = mark.note.clone();
        let field = ui.add(
            TextEdit::multiline(&mut note)
                .hint_text("Add a note")
                .desired_rows(2)
                .desired_width(f32::INFINITY),
        );
        field.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::Role::TextInput,
                true,
                format!("Note for page {}", mark.page + 1),
            )
        });
        if note != mark.note {
            send(out, MarkMsg::EditNote(mark.id, note));
        }
        ui.separator();
    });
}

/// Every mark of the file, in page order.
pub fn list(view: &PdfView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if view.marks.is_empty() {
        ui.weak("No highlights or notes yet. Select text and press Highlight selection, or press Place note.");
        return;
    }
    let mut order: Vec<usize> = (0..view.marks.len()).collect();
    order.sort_by_key(|i| (view.marks[*i].page, view.marks[*i].id));
    for at in order {
        entry(view, at, &view.marks[at], ui, out);
    }
}
