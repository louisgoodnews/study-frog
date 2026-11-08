//! The PDF export dialog view with layout options, live preview, and file destination picker.

use eframe::egui::{self, Key, RichText, Role, WidgetInfo};
use studyfrog_pdf::write::PageSize;
use studyfrog_service::pdf_export::ExportTarget;

use crate::modal::ModalMsg;
use crate::msg::Msg;
use crate::pdf_export_types::{
    PdfExportMsg, PdfExportState, margins_narrow, margins_normal, margins_wide,
};
use crate::pdf_types::PdfMsg;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::icons::{Glyph, icon_button};
use crate::view::widgets::text_field;

fn send(out: &mut Vec<Msg>, m: PdfExportMsg) {
    out.push(Msg::Pdf(PdfMsg::Export(m)));
}

fn target_label(state: &State, target: &ExportTarget) -> String {
    match target {
        ExportTarget::SingleItem(uid) => {
            let title = state
                .library
                .items
                .iter()
                .find(|i| i.uid == *uid)
                .map(|i| i.row.title.as_str())
                .unwrap_or("item");
            format!("Item: {title}")
        }
        ExportTarget::Stack(folder) => {
            if folder.is_empty() {
                "Entire Library".to_string()
            } else {
                format!("Stack: {folder}")
            }
        }
        ExportTarget::Selection(uids) => {
            format!("{} items selected", uids.len())
        }
    }
}

fn chip(ui: &mut egui::Ui, selected: bool, label: &str, acc_name: &str) -> bool {
    let r = ui.selectable_label(selected, label);
    r.widget_info(|| WidgetInfo::labeled(Role::Button, true, acc_name));
    r.clicked()
}

fn options_row(form: &PdfExportState, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal_wrapped(|ui| {
        ui.label("Page size:");
        if chip(
            ui,
            form.options.page_size == PageSize::A4,
            "A4",
            "Page size: A4",
        ) {
            send(out, PdfExportMsg::SetPageSize(PageSize::A4));
        }
        if chip(
            ui,
            form.options.page_size == PageSize::Letter,
            "US Letter",
            "Page size: US Letter",
        ) {
            send(out, PdfExportMsg::SetPageSize(PageSize::Letter));
        }

        ui.add_space(16.0);
        ui.label("Margins:");
        let m = &form.options.margins;
        if chip(ui, m.top == 54.0, "Normal", "Margins: Normal 54 points") {
            send(out, PdfExportMsg::SetMargins(margins_normal()));
        }
        if chip(ui, m.top == 36.0, "Narrow", "Margins: Narrow 36 points") {
            send(out, PdfExportMsg::SetMargins(margins_narrow()));
        }
        if chip(ui, m.top == 72.0, "Wide", "Margins: Wide 72 points") {
            send(out, PdfExportMsg::SetMargins(margins_wide()));
        }

        ui.add_space(16.0);
        ui.label("Font size:");
        for sz in [9.0, 10.0, 11.0, 12.0] {
            let label = format!("{sz:.0} pt");
            let name = format!("Font size: {sz:.0} points");
            if chip(ui, (form.options.font_size - sz).abs() < 0.1, &label, &name) {
                send(out, PdfExportMsg::SetFontSize(sz));
            }
        }
    });
}

fn checkboxes_row(form: &PdfExportState, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal_wrapped(|ui| {
        let mut tp = form.options.title_page;
        let r1 = ui.checkbox(&mut tp, "Title page");
        r1.widget_info(|| WidgetInfo::labeled(Role::CheckBox, true, "Include title page"));
        if r1.changed() {
            send(out, PdfExportMsg::SetTitlePage(tp));
        }

        let mut toc = form.options.table_of_contents;
        let r2 = ui.checkbox(&mut toc, "Table of contents");
        r2.widget_info(|| WidgetInfo::labeled(Role::CheckBox, true, "Include table of contents"));
        if r2.changed() {
            send(out, PdfExportMsg::SetToc(toc));
        }

        let mut num = form.options.page_numbers;
        let r3 = ui.checkbox(&mut num, "Page numbers");
        r3.widget_info(|| WidgetInfo::labeled(Role::CheckBox, true, "Include page numbers"));
        if r3.changed() {
            send(out, PdfExportMsg::SetPageNumbers(num));
        }

        let mut ans = form.options.show_answers;
        let r4 = ui.checkbox(&mut ans, "Card/question answers");
        r4.widget_info(|| {
            WidgetInfo::labeled(Role::CheckBox, true, "Include card and question answers")
        });
        if r4.changed() {
            send(out, PdfExportMsg::SetShowAnswers(ans));
        }
    });
}

fn destination_row(form: &PdfExportState, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal(|ui| {
        let text = form.destination.clone();
        if let Some(t) = text_field(ui, "Destination file (.pdf)", &text, "", 1) {
            send(out, PdfExportMsg::EditDestination(t));
        }
        let b = ui.button("Browse…");
        b.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Browse for destination file"));
        if b.clicked() {
            send(out, PdfExportMsg::BrowseDestination);
        }
    });
}

fn bottom_actions(form: &PdfExportState, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal(|ui| {
        let cancel = ui.button("Cancel");
        cancel.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Cancel PDF export"));
        if cancel.clicked() {
            out.push(Msg::Modal(ModalMsg::Close));
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let can_export = !form.busy && !form.destination.trim().is_empty();
            let label = if form.busy {
                "Exporting…"
            } else {
                "Export PDF"
            };
            let submit = ui.add_enabled(can_export, egui::Button::new(label));
            submit.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Export PDF"));
            if (submit.clicked() || (ui.input(|i| i.key_pressed(Key::Enter)) && can_export))
                && !form.busy
            {
                send(out, PdfExportMsg::Submit);
            }
        });
    });
}

/// Draw the PDF export dialog.
pub fn draw(state: &State, form: &PdfExportState, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let p = palette_of(ui);
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Export to PDF")
                .strong()
                .size(crate::typo::size(ui, crate::typo::Tier::Heading)),
        );
        ui.label(
            RichText::new(format!("({})", target_label(state, &form.target))).color(p.text_weak),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if icon_button(ui, Glyph::Close, "Close", false).clicked() {
                out.push(Msg::Modal(ModalMsg::Close));
            }
        });
    });

    ui.add_space(12.0);
    options_row(form, ui, out);
    ui.add_space(8.0);
    checkboxes_row(form, ui, out);
    ui.add_space(8.0);
    destination_row(form, ui, out);

    if let Some(status) = &form.status {
        ui.add_space(4.0);
        ui.label(RichText::new(status).color(p.accent));
    }

    ui.add_space(12.0);
    super::modal_pdf_preview::draw_preview(form, ui);

    ui.add_space(16.0);
    bottom_actions(form, ui, out);
}
