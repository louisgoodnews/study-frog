//! The viewer's page controls and the zoom presets, kept out of `pdf.rs` to keep it readable.
use super::*;
use crate::pdf_types::Zoom;

pub(super) fn navigation(view: &PdfView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let pages = view.info.as_ref().map_or(0, |i| i.page_count);
    if ui
        .button("Previous")
        .on_hover_text("Goes to the previous one.")
        .clicked()
    {
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
    if ui
        .button("Next")
        .on_hover_text("Goes to the next one.")
        .clicked()
    {
        msg(out, PdfMsg::Next);
    }
}

/// The zoom as the menu shows it: a fit, or a percentage.
pub(super) fn zoom_name(zoom: Zoom) -> String {
    match zoom {
        Zoom::FitWidth | Zoom::FitPage => "Fit".to_owned(),
        Zoom::Percent(p) => format!("{p} %"),
    }
}

pub(super) fn fit_presets(view: &PdfView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let fit = view.zoom == Zoom::FitWidth;
    if ui.add(egui::Button::selectable(fit, "Fit width")).clicked() {
        msg(out, PdfMsg::SetZoom(Zoom::FitWidth));
        ui.close();
    }
    let page = view.zoom == Zoom::FitPage;
    if ui.add(egui::Button::selectable(page, "Fit page")).clicked() {
        msg(out, PdfMsg::SetZoom(Zoom::FitPage));
        ui.close();
    }
}
