//! The PDF screen: a toolbar, a side panel (outline or page thumbnails) and the page.

use eframe::egui::{self, Panel, RichText, TextEdit};

use crate::msg::Msg;
use crate::pdf_arrange_types::ArrangeMsg;
use crate::pdf_link_types::LinkMsg;
use crate::pdf_select::selected_text;
use crate::pdf_types::{PdfMsg, PdfView, Side, Zoom};
use crate::state::State;
use crate::view::{pdf_arrange, pdf_notes, pdf_page, pdf_side};

fn msg(out: &mut Vec<Msg>, m: PdfMsg) {
    out.push(Msg::Pdf(m));
}

fn navigation(view: &PdfView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let pages = view.info.as_ref().map_or(0, |i| i.page_count);
    if ui.button("Previous").clicked() {
        msg(out, PdfMsg::Previous);
    }
    let mut shown = view.page + 1;
    let drag = egui::DragValue::new(&mut shown)
        .range(1..=pages.max(1))
        .speed(0.2);
    let label = ui.add(drag);
    label.widget_info(|| egui::WidgetInfo::labeled(egui::Role::SpinButton, true, "Page number"));
    if shown != view.page + 1 {
        msg(out, PdfMsg::GoTo(shown - 1));
    }
    ui.label(format!("of {pages}"));
    if ui.button("Next").clicked() {
        msg(out, PdfMsg::Next);
    }
}

fn zoom_controls(view: &PdfView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if ui.button("-").on_hover_text("Zoom out").clicked() {
        msg(out, PdfMsg::ZoomOut);
    }
    let name = match view.zoom {
        Zoom::FitWidth | Zoom::FitPage => "Fit".to_owned(),
        Zoom::Percent(p) => format!("{p} %"),
    };
    ui.label(name);
    if ui.button("+").on_hover_text("Zoom in").clicked() {
        msg(out, PdfMsg::ZoomIn);
    }
    let fit = view.zoom == Zoom::FitWidth;
    if ui.add(egui::Button::selectable(fit, "Fit width")).clicked() {
        msg(out, PdfMsg::SetZoom(Zoom::FitWidth));
    }
    let page = view.zoom == Zoom::FitPage;
    if ui.add(egui::Button::selectable(page, "Fit page")).clicked() {
        msg(out, PdfMsg::SetZoom(Zoom::FitPage));
    }
}

fn search(view: &PdfView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let mut text = view.query.clone();
    let field = ui.add(
        TextEdit::singleline(&mut text)
            .hint_text("Find in this PDF")
            .desired_width(160.0),
    );
    field
        .widget_info(|| egui::WidgetInfo::labeled(egui::Role::TextInput, true, "Find in this PDF"));
    if text != view.query {
        msg(out, PdfMsg::Query(text));
    }
    if view.query.trim().is_empty() {
        return;
    }
    if ui.button("Previous hit").clicked() {
        msg(out, PdfMsg::PreviousHit);
    }
    if ui.button("Next hit").clicked() {
        msg(out, PdfMsg::NextHit);
    }
    ui.weak(format!("{} pages", view.found.len()));
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
            .clicked()
        {
            msg(out, PdfMsg::ShowSide(side));
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
    let button = ui.add_enabled(!text.is_empty(), egui::Button::new("Copy selection"));
    if button.clicked() {
        ui.ctx().copy_text(text);
    }
}

fn toolbar(view: &PdfView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal_wrapped(|ui| {
        if ui.button("Back to library").clicked() {
            msg(out, PdfMsg::Close);
        }
        ui.label(RichText::new(crate::pdf_names::label(&view.path, &view.title)).strong());
        ui.separator();
        navigation(view, ui, out);
        ui.separator();
        zoom_controls(view, ui, out);
        ui.separator();
        search(view, ui, out);
        ui.separator();
        copy_button(view, ui);
        if ui.button("Arrange pages ...").clicked() {
            msg(out, PdfMsg::Arrange(ArrangeMsg::Start));
        }
        ui.separator();
        side_buttons(view, ui, out);
    });
}

/// Draw the viewer for the open PDF.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let stacks = state
        .vault
        .as_ref()
        .map_or(&[][..], |v| v.overview.stacks.as_slice());
    let Some(view) = &state.pdf else {
        ui.weak("No PDF is open. Choose one from the search or the library.");
        return;
    };
    if let Some(error) = &view.error {
        ui.colored_label(
            ui.visuals().error_fg_color,
            format!("This PDF cannot be shown: {error}"),
        );
    }
    toolbar(view, ui, out);
    pdf_notes::bar(view, ui, out);
    link_question(view, ui, out);
    ui.separator();
    if view.info.is_none() {
        if view.error.is_none() {
            ui.weak("Opening the file ...");
        }
        return;
    }
    if !view.scanned.is_empty() {
        let n = view.scanned.len();
        ui.weak(format!(
            "{n} {} of this PDF only {} pictures, without text. {} cannot be searched, selected or turned into notes (reading text from pictures is not part of StudyFrog yet).",
            if n == 1 { "page" } else { "pages" },
            if n == 1 { "holds" } else { "hold" },
            if n == 1 { "It" } else { "They" },
        ));
    }
    if view.side != Side::Closed {
        Panel::left("pdf-side")
            .default_size(170.0)
            .show(ui, |ui| pdf_side::draw(view, stacks, ui, out));
    }
    egui::CentralPanel::default().show(ui, |ui| match &view.arrange {
        Some(a) => pdf_arrange::draw(view, a, ui, out),
        None => pdf_page::draw(view, ui, out),
    });
}

/// The question that comes before a web address is opened.
fn link_question(view: &PdfView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(url) = &view.pending_link else {
        return;
    };
    ui.horizontal_wrapped(|ui| {
        ui.label(format!("This PDF links to {url}. Open it in your browser?"));
        if ui.button("Open").clicked() {
            ui.ctx().open_url(egui::OpenUrl::new_tab(url));
            out.push(Msg::Pdf(PdfMsg::Link(LinkMsg::Approve)));
        }
        if ui.button("Do not open").clicked() {
            out.push(Msg::Pdf(PdfMsg::Link(LinkMsg::Dismiss)));
        }
    });
}
