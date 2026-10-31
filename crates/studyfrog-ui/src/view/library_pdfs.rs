//! PDFs in the library and on the home page: a list per stack with page counts, adding, renaming.

use eframe::egui::{self, RichText, TextEdit};
use studyfrog_store::pdf_index::PdfRow;

use crate::msg::Msg;
use crate::pdf_library_types::LibraryMsg;
use crate::pdf_types::PdfMsg;
use crate::state::State;
use crate::view::kit;

fn send(out: &mut Vec<Msg>, m: LibraryMsg) {
    out.push(Msg::Pdf(PdfMsg::Library(m)));
}

fn pages_word(row: &PdfRow) -> String {
    match (&row.error, row.pages) {
        (Some(_), _) => "cannot be read".to_owned(),
        (None, 1) => "1 page".to_owned(),
        (None, n) => format!("{n} pages"),
    }
}

fn file_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

fn rename_row(state: &State, row: &PdfRow, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some((_, name)) = state
        .pdf_library
        .rename
        .as_ref()
        .filter(|(p, _)| *p == row.path)
    else {
        return;
    };
    let mut text = name.clone();
    ui.horizontal(|ui| {
        let label = ui.label("New file name");
        ui.add(TextEdit::singleline(&mut text).desired_width(260.0))
            .labelled_by(label.id);
        if ui.button("Rename").clicked() {
            send(out, LibraryMsg::DoRename);
        }
        if ui.button("Cancel").clicked() {
            send(out, LibraryMsg::CancelRename);
        }
    });
    if &text != name {
        send(out, LibraryMsg::RenameEdit(text));
    }
}

fn row(state: &State, row: &PdfRow, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal(|ui| {
        kit::pill(ui, "pdf", crate::theme::palette_of(ui).accent);
        let label = format!("{} ({})", row.title, pages_word(row));
        if ui.link(label).on_hover_text(&row.path).clicked() {
            out.push(Msg::Pdf(PdfMsg::Open(row.path.clone(), 0)));
        }
        ui.weak(file_name(&row.path));
        if ui.small_button("Rename").clicked() {
            send(out, LibraryMsg::StartRename(row.path.clone()));
        }
    });
    rename_row(state, row, ui, out);
}

/// The PDFs that sit directly in the stack's folder, with the button that adds one.
pub fn stack_section(state: &State, folder: &str, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.separator();
    ui.horizontal(|ui| {
        ui.strong("PDF files");
        if ui.button("Add PDF ...").clicked() {
            send(out, LibraryMsg::AddTo(folder.to_owned()));
        }
    });
    let prefix = format!("{folder}/");
    let here = state.pdf_library.rows.iter().filter(|r| {
        r.path
            .strip_prefix(&prefix)
            .is_some_and(|rest| !rest.contains('/'))
    });
    let mut any = false;
    for r in here {
        any = true;
        row(state, r, ui, out);
    }
    if !any {
        ui.weak("No PDF files in this stack yet.");
    }
}

/// The PDFs that changed most recently, for the home page.
pub fn recent_card(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let mut rows: Vec<&PdfRow> = state
        .pdf_library
        .rows
        .iter()
        .filter(|r| r.error.is_none())
        .collect();
    if rows.is_empty() {
        return;
    }
    rows.sort_by_key(|r| std::cmp::Reverse(r.mtime_ns));
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(RichText::new("PDF files").strong());
        for r in rows.into_iter().take(5) {
            let label = format!("{} ({})", r.title, pages_word(r));
            if ui.link(label).on_hover_text(&r.path).clicked() {
                out.push(Msg::Pdf(PdfMsg::Open(r.path.clone(), 0)));
            }
        }
    });
}
