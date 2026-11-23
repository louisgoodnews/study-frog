//! The PDF screen: a toolbar, a side panel (outline or page thumbnails) and the page.

mod more_menu;
mod toolbar_nav;
use self::more_menu::more;
use self::toolbar_nav::navigation;
use eframe::egui::{self, Panel, RichText, TextEdit};
use studyfrog_core::count::counted;

use crate::msg::Msg;
use crate::pdf_link_types::LinkMsg;
use crate::pdf_types::{PdfMsg, PdfView, Side};
use crate::state::State;
use crate::view::{pdf_arrange, pdf_notes, pdf_page, pdf_side};

fn msg(out: &mut Vec<Msg>, m: PdfMsg) {
    out.push(Msg::Pdf(m));
}

fn search(view: &PdfView, ui: &mut egui::Ui, out: &mut Vec<Msg>, requested: bool) {
    let mut text = view.query.clone();
    let field = ui
        .add(
            TextEdit::singleline(&mut text)
                .hint_text(crate::view::search_focus::PDF_FIND)
                .desired_width(160.0),
        )
        .on_hover_text(crate::view::tips::with_key(
            "Finds the text in this PDF.",
            "Ctrl+F or /",
        ));
    if crate::view::search_focus::focus_field(
        ui.ctx(),
        &field,
        view.query.chars().count(),
        requested,
    ) {
        crate::view::search_focus::taken(out, crate::view::search_focus::PDF_FIND);
    }
    field.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::Role::TextInput,
            true,
            crate::view::search_focus::named(crate::view::search_focus::PDF_FIND),
        )
    });
    if text != view.query {
        msg(out, PdfMsg::Query(text));
    }
    if view.query.trim().is_empty() {
        return;
    }
    ui.weak(counted(view.found.len(), "page", "pages"));
}

/// The header: Back, the page, the find field and More. Zoom, search hits, copy, arrange and the
/// side panels are in More.
fn toolbar(view: &PdfView, ui: &mut egui::Ui, out: &mut Vec<Msg>, requested: bool) {
    ui.horizontal_wrapped(|ui| {
        if ui
            .button("Back to library")
            .on_hover_text("Goes back to the library.")
            .clicked()
        {
            msg(out, PdfMsg::Close);
        }
        ui.label(RichText::new(crate::pdf_names::label(&view.path, &view.title)).strong());
        ui.separator();
        navigation(view, ui, out);
        ui.separator();
        search(view, ui, out, requested);
        ui.separator();
        more(view, ui, out);
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
    toolbar(view, ui, out, state.chrome.search_focus);
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
        if ui
            .button("Open")
            .on_hover_text("Opens the link in your browser.")
            .clicked()
        {
            ui.ctx().open_url(egui::OpenUrl::new_tab(url));
            out.push(Msg::Pdf(PdfMsg::Link(LinkMsg::Approve)));
        }
        if ui
            .button("Do not open")
            .on_hover_text("Closes without opening the file.")
            .clicked()
        {
            out.push(Msg::Pdf(PdfMsg::Link(LinkMsg::Dismiss)));
        }
    });
}
