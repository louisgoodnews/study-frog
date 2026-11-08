//! Rearranging pages into a new file: a grid of the pages to turn, move and drop, and the form that writes it.

use eframe::egui::{self, Rect, ScrollArea, Sense, TextEdit, Vec2};

use crate::msg::Msg;
use crate::pdf_arrange_types::{Arrange, ArrangeMsg, Prop};
use crate::pdf_types::{PageKey, PdfMsg, PdfView, THUMB_SCALE_MILLI};
use crate::view::pdf_page::{mark_used, picture_for, texture};

const CELL: f32 = 130.0;

fn send(out: &mut Vec<Msg>, m: ArrangeMsg) {
    out.push(Msg::Pdf(PdfMsg::Arrange(m)));
}

fn actions(a: &Arrange, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal_wrapped(|ui| {
        let some = a.selected.is_some();
        for (label, msg) in [
            ("Turn left", ArrangeMsg::Rotate(-90)),
            ("Turn right", ArrangeMsg::Rotate(90)),
            ("Move earlier", ArrangeMsg::Move(-1)),
            ("Move later", ArrangeMsg::Move(1)),
            ("Delete page", ArrangeMsg::Delete),
        ] {
            if ui
                .add_enabled(some && !a.busy, egui::Button::new(label))
                .clicked()
            {
                send(out, msg);
            }
        }
        ui.separator();
        if ui
            .add_enabled(!a.undo.is_empty(), egui::Button::new("Undo"))
            .clicked()
        {
            send(out, ArrangeMsg::Undo);
        }
        if ui.button("Start over").clicked() {
            send(out, ArrangeMsg::Reset);
        }
        if ui.button("Add a PDF ...").clicked() {
            send(out, ArrangeMsg::AddFile);
        }
    });
}

fn cell(view: &PdfView, a: &Arrange, at: usize, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let step = a.plan[at];
    let (rect, response) = ui.allocate_exact_size(Vec2::new(CELL, CELL + 18.0), Sense::click());
    let area = Rect::from_min_size(rect.min, Vec2::splat(CELL));
    let size = view
        .info
        .as_ref()
        .filter(|_| step.source == 0)
        .and_then(|i| i.pages.get(step.page));
    let fit = size.map_or(Vec2::splat(CELL * 0.8), |s| {
        let k = (CELL * 0.9) / s.width.max(s.height).max(1.0);
        Vec2::new(s.width * k, s.height * k)
    });
    if ui.is_rect_visible(rect) {
        let key = PageKey {
            page: step.page,
            scale_milli: THUMB_SCALE_MILLI,
        };
        match picture_for(view, key).filter(|_| step.source == 0) {
            Some(px) => {
                let tex = texture(ui.ctx(), &view.path, key, &px);
                let image = egui::Image::new(&tex)
                    .fit_to_exact_size(fit)
                    .rotate((step.rotate as f32).to_radians(), Vec2::splat(0.5));
                image.paint_at(ui, Rect::from_center_size(area.center(), fit));
                mark_used(view, key, out);
            }
            None => {
                ui.painter().rect_filled(
                    Rect::from_center_size(area.center(), fit),
                    2.0,
                    ui.visuals().extreme_bg_color,
                );
                if step.source == 0 {
                    out.push(Msg::Pdf(PdfMsg::Need(key)));
                }
            }
        }
    }
    if a.selected == Some(at) {
        ui.painter().rect_stroke(
            area,
            4.0,
            ui.visuals().selection.stroke,
            egui::StrokeKind::Inside,
        );
    }
    let from = if step.source == 0 {
        String::new()
    } else {
        format!(" (file {})", step.source + 1)
    };
    let turn = if step.rotate == 0 {
        String::new()
    } else {
        format!(", turned {}", step.rotate)
    };
    let text = format!("{}: page {}{from}{turn}", at + 1, step.page + 1);
    ui.painter().text(
        rect.left_bottom() + Vec2::new(4.0, -2.0),
        egui::Align2::LEFT_BOTTOM,
        &text,
        crate::typo::font(ui, crate::typo::Tier::Caption),
        ui.visuals().text_color(),
    );
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::Role::Button,
            true,
            a.selected == Some(at),
            text.clone(),
        )
    });
    if response.clicked() {
        send(out, ArrangeMsg::Select(at));
    }
}

fn field(ui: &mut egui::Ui, label: &str, value: &str) -> Option<String> {
    let mut text = value.to_owned();
    ui.horizontal(|ui| {
        let name = ui.label(label);
        ui.add(TextEdit::singleline(&mut text).desired_width(280.0))
            .labelled_by(name.id);
    });
    (text != value).then_some(text)
}

fn form(a: &Arrange, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if let Some(t) = field(ui, "Name of the new file", &a.name) {
        send(out, ArrangeMsg::Name(t));
    }
    for (prop, label, i) in [
        (Prop::Title, "Title", 0),
        (Prop::Author, "Author", 1),
        (Prop::Subject, "Subject", 2),
    ] {
        if let Some(t) = field(ui, label, &a.props[i]) {
            send(out, ArrangeMsg::Property(prop, t));
        }
    }
    let mut marks = a.include_marks;
    ui.checkbox(&mut marks, "Write my highlights and notes into the copy");
    if marks != a.include_marks {
        send(out, ArrangeMsg::IncludeMarks(marks));
    }
    ui.horizontal(|ui| {
        if ui
            .add_enabled(!a.busy, egui::Button::new("Save as new file"))
            .clicked()
        {
            send(out, ArrangeMsg::Save);
        }
        if ui.button("Cancel").clicked() {
            send(out, ArrangeMsg::Cancel);
        }
    });
    ui.separator();
    if let Some(t) = field(
        ui,
        "Split the original at pages (for example 1-3, 4-9)",
        &a.ranges,
    ) {
        send(out, ArrangeMsg::Ranges(t));
    }
    if ui
        .add_enabled(
            !a.busy && !a.ranges.trim().is_empty(),
            egui::Button::new("Split into files"),
        )
        .clicked()
    {
        send(out, ArrangeMsg::Split);
    }
    if let Some(m) = &a.message {
        ui.colored_label(ui.visuals().error_fg_color, m);
    }
    if a.busy {
        ui.weak("Writing ...");
    }
}

/// Draw the rearranging page of the PDF screen.
pub fn draw(view: &PdfView, a: &Arrange, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.heading("Arrange pages into a new file");
    ui.weak("The original is never changed. Turn, move or drop pages, add another PDF, then save under a new name.");
    actions(a, ui, out);
    ScrollArea::vertical()
        .id_salt("pdf-arrange-scroll")
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for at in 0..a.plan.len() {
                    cell(view, a, at, ui, out);
                }
            });
            ui.separator();
            form(a, ui, out);
        });
}
