//! The viewer's "More…" menu: zoom, search hits, copy, arrange pages and the side panels. The
//! header keeps only the core actions; everything here is one click further.
use super::toolbar_nav::{fit_presets, zoom_name};
use super::*;
use crate::pdf_arrange_types::ArrangeMsg;
use crate::pdf_select::selected_text;

pub(super) fn more(view: &PdfView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.menu_button("More…", |ui| {
        zoom_items(view, ui, out);
        ui.separator();
        hit_items(view, ui, out);
        ui.separator();
        copy_button(view, ui);
        arrange_button(ui, out);
        ui.separator();
        ui.menu_button("Panels…", |ui| side_buttons(view, ui, out))
            .response
            .on_hover_text("Shows or hides the side panels: outline, pages, notes and extracts.");
    })
    .response
    .on_hover_text("Zoom, search hits, copy, arrange pages and the side panels.");
}

fn zoom_items(view: &PdfView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.weak(format!("Zoom: {}", zoom_name(view.zoom)));
    if ui
        .button("Zoom in")
        .on_hover_text("Makes the page larger.")
        .clicked()
    {
        msg(out, PdfMsg::ZoomIn);
    }
    if ui
        .button("Zoom out")
        .on_hover_text("Makes the page smaller.")
        .clicked()
    {
        msg(out, PdfMsg::ZoomOut);
    }
    fit_presets(view, ui, out);
}

fn hit_items(view: &PdfView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let searching = !view.query.trim().is_empty();
    if ui
        .add_enabled(searching, egui::Button::new("Previous hit"))
        .on_hover_text("Goes to the previous match.")
        .clicked()
    {
        msg(out, PdfMsg::PreviousHit);
    }
    if ui
        .add_enabled(searching, egui::Button::new("Next hit"))
        .on_hover_text("Goes to the next match.")
        .clicked()
    {
        msg(out, PdfMsg::NextHit);
    }
}

fn arrange_button(ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if ui
        .button("Arrange pages ...")
        .on_hover_text("Opens the page arranger to reorder or remove pages.")
        .clicked()
    {
        msg(out, PdfMsg::Arrange(ArrangeMsg::Start));
        ui.close();
    }
}

fn side_buttons(view: &PdfView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    for (side, name) in [
        (Side::Outline, "Outline"),
        (Side::Pages, "Pages"),
        (Side::Notes, "Notes"),
        (Side::Extract, "Extract"),
        (Side::Closed, "Hide"),
    ] {
        if ui
            .add(egui::Button::selectable(view.side == side, name))
            .on_hover_text("Shows this panel beside the page.")
            .clicked()
        {
            msg(out, PdfMsg::ShowSide(side));
            ui.close();
        }
    }
}

fn copy_button(view: &PdfView, ui: &mut egui::Ui) {
    let words = view
        .text
        .as_ref()
        .map(|t| t.words.as_slice())
        .unwrap_or(&[]);
    let text = selected_text(words, &view.selection);
    let button = ui
        .add_enabled(!text.is_empty(), egui::Button::new("Copy selection"))
        .on_hover_text("Copies the selected text.");
    if button.clicked() {
        ui.ctx().copy_text(text);
    }
}
