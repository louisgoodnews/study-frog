//! Split-screen PDF viewer layout, contextual jump button, and embedded pane rendering.

use eframe::egui;

use crate::msg::Msg;
pub use crate::pdf_split_ui::{clamp_page_index, close_split, open_crop, set_split_ratio};
use crate::pdf_types::{PageKey, PdfMsg, PdfView, SplitPdfView};
use crate::state::State;
use crate::view::{pdf_page, pdf_paint};

/// Render contextual PDF jump button if `source` is a valid PDF crop anchor URI.
pub fn jump_button(source: &str, ui: &mut egui::Ui, out: &mut Vec<Msg>) -> bool {
    let Some(anchor) = studyfrog_pdf::crop::parse_pdf_crop_uri(source) else {
        return false;
    };
    let p = crate::theme::palette_of(ui);
    let label = format!("📖 Jump to PDF (p. {})", anchor.page);
    let btn = egui::Button::new(egui::RichText::new(label).color(p.accent).strong())
        .stroke(egui::Stroke::new(1.0, p.accent));
    if ui
        .add(btn)
        .on_hover_text("Shows the source of this card in the PDF beside it.")
        .clicked()
    {
        out.push(Msg::Pdf(PdfMsg::OpenCrop(anchor)));
    }
    true
}

/// Renders a contextual PDF Jump button or fallback source link.
pub fn source_link(source: &str, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if jump_button(source, ui, out) {
        return;
    }
    let (path, page) = studyfrog_store::pdf_notes::split_source(source);
    if !path.to_lowercase().ends_with(".pdf") {
        ui.label(format!("Source: {source}"));
        return;
    }
    let label = page.map_or("Source".to_owned(), |p| format!("Source, page {}", p + 1));
    if ui
        .small_button(label)
        .on_hover_text("Opens the source of this card in the PDF.")
        .clicked()
    {
        out.push(Msg::Pdf(PdfMsg::Open(path.to_owned(), page.unwrap_or(0))));
    }
}

/// Handle keyboard shortcuts for split pane resizing and dismissing.
pub fn handle_split_keys(split: &SplitPdfView, ui: &egui::Ui, out: &mut Vec<Msg>) {
    if ui.ctx().egui_wants_keyboard_input() {
        return;
    }
    let alt = |k| ui.input(|i| i.modifiers.alt && i.key_pressed(k));
    if alt(egui::Key::W) || alt(egui::Key::P) {
        out.push(Msg::Pdf(PdfMsg::CloseSplit));
    }
    if alt(egui::Key::OpenBracket) || alt(egui::Key::Minus) {
        out.push(Msg::Pdf(PdfMsg::SetSplitRatio(split.split_ratio - 0.05)));
    }
    if alt(egui::Key::CloseBracket) || alt(egui::Key::Plus) || alt(egui::Key::Equals) {
        out.push(Msg::Pdf(PdfMsg::SetSplitRatio(split.split_ratio + 0.05)));
    }
}

fn draw_split_container<F>(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>, draw_left: F)
where
    F: FnOnce(&State, &mut egui::Ui, &mut Vec<Msg>),
{
    handle_split_keys(&state.split_pdf, ui, out);
    let total_w = ui.available_width();
    let ratio = state.split_pdf.split_ratio.clamp(0.2, 0.8);
    let left_w = (total_w * ratio).clamp(240.0, total_w - 200.0);
    let right_w = (total_w - left_w - 10.0).max(180.0);
    let height = ui.available_height();

    ui.horizontal_top(|ui| {
        let down = egui::Layout::top_down(egui::Align::Min);
        ui.allocate_ui_with_layout(egui::vec2(left_w, height), down, |ui| {
            draw_left(state, ui, out);
        });
        ui.separator();
        ui.allocate_ui_with_layout(egui::vec2(right_w, height), down, |ui| {
            draw_embedded_pdf_pane(state, ui, out);
        });
    });
}

/// Render side-by-side split view with rehearsal card on left and PDF viewer on right.
pub fn draw_split_rehearse(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    draw_split_container(state, ui, out, super::run::draw);
}

/// Render side-by-side split view with document editor on left and PDF viewer on right.
pub fn draw_split_editor(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    draw_split_container(state, ui, out, super::editor::draw);
}

fn draw_embedded_pdf_pane(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(view) = &state.pdf else {
        ui.weak("Opening PDF viewer…");
        return;
    };
    if let Some(err) = &view.error {
        ui.colored_label(
            ui.visuals().error_fg_color,
            format!("Unable to load PDF: {err}"),
        );
        if ui
            .button("Close split")
            .on_hover_text("Closes the split view.")
            .clicked()
        {
            out.push(Msg::Pdf(PdfMsg::CloseSplit));
        }
        return;
    }
    draw_split_header(state, view, ui, out);
    ui.separator();
    draw_embedded_page(state, view, ui, out);
}

fn draw_split_header(state: &State, view: &PdfView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let pages = view.info.as_ref().map_or(0, |i| i.page_count);
    let p = crate::theme::palette_of(ui);
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(&view.title).strong().color(p.accent));
        ui.separator();
        if ui
            .small_button("◀")
            .on_hover_text("Previous page")
            .clicked()
        {
            out.push(Msg::Pdf(PdfMsg::Previous));
        }
        ui.label(format!("p. {} / {}", view.page + 1, pages.max(1)));
        if ui.small_button("▶").on_hover_text("Next page").clicked() {
            out.push(Msg::Pdf(PdfMsg::Next));
        }
        if let Some(anchor) = &state.split_pdf.anchor {
            let target_p = (anchor.page.saturating_sub(1)) as usize;
            if view.page != target_p
                && ui
                    .small_button(format!("📍 p. {}", anchor.page))
                    .on_hover_text("Jump to highlight")
                    .clicked()
            {
                out.push(Msg::Pdf(PdfMsg::GoTo(target_p)));
            }
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .small_button("✕")
                .on_hover_text("Close split view (Alt+W)")
                .clicked()
            {
                out.push(Msg::Pdf(PdfMsg::CloseSplit));
            }
            if ui
                .selectable_label(state.split_pdf.split_ratio >= 0.65, "70%")
                .clicked()
            {
                out.push(Msg::Pdf(PdfMsg::SetSplitRatio(0.7)));
            }
            if ui
                .selectable_label((state.split_pdf.split_ratio - 0.5).abs() < 0.1, "50%")
                .clicked()
            {
                out.push(Msg::Pdf(PdfMsg::SetSplitRatio(0.5)));
            }
            if ui
                .selectable_label(state.split_pdf.split_ratio <= 0.35, "30%")
                .clicked()
            {
                out.push(Msg::Pdf(PdfMsg::SetSplitRatio(0.3)));
            }
        });
    });
}

fn draw_embedded_page(state: &State, view: &PdfView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(size) = view
        .info
        .as_ref()
        .and_then(|i| i.pages.get(view.page))
        .copied()
    else {
        ui.weak("Loading page…");
        return;
    };
    pdf_page::prune(ui.ctx(), view);
    let (milli, logical) = pdf_page::layout(
        view,
        ui.available_size(),
        ui.ctx().pixels_per_point(),
        (size.width, size.height),
        ui.input(|i| i.max_texture_side),
    );
    let key = PageKey {
        page: view.page,
        scale_milli: milli,
    };
    egui::ScrollArea::both()
        .id_salt("pdf-split-page-scroll")
        .show(ui, |ui| {
            let (rect, response) = ui.allocate_exact_size(
                egui::Vec2::new(size.width, size.height) * logical,
                egui::Sense::click_and_drag(),
            );
            match pdf_page::picture_for(view, key) {
                Some(px) => {
                    let tex = pdf_page::texture(ui.ctx(), &view.path, key, &px);
                    egui::Image::new(&tex).paint_at(ui, rect);
                    pdf_page::mark_used(view, key, out);
                }
                None => {
                    ui.painter()
                        .rect_filled(rect, 0.0, ui.visuals().extreme_bg_color);
                    if view.requested.contains(&key) {
                        ui.ctx().request_repaint();
                    }
                }
            }
            if !view.pictures.iter().any(|(k, _)| *k == key) {
                out.push(Msg::Pdf(PdfMsg::Need(key)));
            }
            pdf_paint::paint_marks(view, ui, rect, logical);
            pdf_paint::paint_words(view, ui, rect, logical);
            pdf_paint::paint_links(view, ui, rect, logical);
            if let Some(anchor) = &state.split_pdf.anchor
                && (anchor.page.saturating_sub(1)) as usize == view.page
            {
                pdf_paint::paint_crop_highlight(
                    ui,
                    anchor,
                    (size.width, size.height),
                    rect,
                    logical,
                    (state.split_pdf.flash_start_now, state.now),
                );
            }
            let _ = response;
        });
}
