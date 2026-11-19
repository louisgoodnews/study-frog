//! The Import tab of the Universal Import Hub: drop zone, pickers, format badge, destination picker, preview and commit.

use std::path::Path;

use eframe::egui::{self, CornerRadius, Margin, RichText, Stroke, vec2};
use studyfrog_service::xfer_adapters::{SourceFormat, format_label as service_format_label};

use crate::import_hub::{HubMsg, ImportDestination};
use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;
use crate::theme_space::rhythm_of;
use crate::transfer_ui::TransferMsg;
use crate::view::kit;
use crate::view::widgets::text_field;

/// Format auto-detected from a file or folder path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectedFormat {
    /// Built-in service format.
    BuiltIn(SourceFormat),
    /// Registered extension importer.
    Extension(String),
    /// Unrecognized extension.
    Unknown,
}

/// Whether `fmt` represents a PDF document.
pub fn is_pdf_format(fmt: Option<&DetectedFormat>) -> bool {
    matches!(fmt, Some(DetectedFormat::BuiltIn(SourceFormat::Pdf)))
}

/// Auto-detect the source format from `path`, `is_folder`, and registered extension importers.
pub fn detect_format(
    path: &str,
    is_folder: bool,
    importers: &[(String, String, Vec<String>)],
) -> Option<DetectedFormat> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return None;
    }
    let name = trimmed.rsplit(['/', '\\']).next().unwrap_or(trimmed);
    if is_folder || trimmed.ends_with(['/', '\\']) || !name.contains('.') {
        return Some(DetectedFormat::BuiltIn(SourceFormat::ObsidianFolder));
    }
    if let Some(sf) = studyfrog_service::xfer_adapters::detect_format(Path::new(trimmed)) {
        return Some(DetectedFormat::BuiltIn(sf));
    }
    let ext = name
        .rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .unwrap_or_default();
    for (_, label, exts) in importers {
        if exts.iter().any(|e| e.eq_ignore_ascii_case(&ext)) {
            return Some(DetectedFormat::Extension(label.clone()));
        }
    }
    Some(DetectedFormat::Unknown)
}

/// Human-readable label for a [`DetectedFormat`].
pub fn format_label(fmt: &DetectedFormat) -> String {
    match fmt {
        DetectedFormat::BuiltIn(sf) => service_format_label(*sf).to_owned(),
        DetectedFormat::Extension(name) => format!("Extension: {name}"),
        DetectedFormat::Unknown => "Unknown format".to_owned(),
    }
}

fn send(out: &mut Vec<Msg>, m: TransferMsg) {
    out.push(Msg::Library(crate::library_ui::LibraryMsg::Transfer(m)));
}

fn drop_zone(ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let hovering = ui.ctx().input(|i| !i.raw.hovered_files.is_empty());
    let p = palette_of(ui);
    let pad = rhythm_of(ui).item.round() as i8;
    let fill = if hovering { p.surface_hover } else { p.surface };
    let stroke = Stroke::new(
        if hovering { 2.0 } else { 1.0 },
        if hovering { p.accent } else { p.border },
    );
    egui::Frame::new()
        .fill(fill)
        .stroke(stroke)
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::symmetric(pad, 6))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                ui.strong(if hovering {
                    "Release to select file…"
                } else {
                    "Drop a file or folder here"
                });
                ui.weak("· CSV, TSV, Markdown, Anki (.apkg), PDF, JSON, TOML, RON, Zip");
            });
        });
    let dropped = ui.ctx().input(|i| i.raw.dropped_files.clone());
    if let Some(first) = dropped.first() {
        let path = first.path();
        if path.as_os_str().is_empty() {
            send(
                out,
                TransferMsg::Hub(Box::new(HubMsg::DropError(
                    "Dropped item had no readable file path.".to_owned(),
                ))),
            );
        } else {
            send(
                out,
                TransferMsg::Hub(Box::new(HubMsg::Dropped(path.display().to_string()))),
            );
        }
    }
}

fn pdf_commit_bar(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let t = &state.transfer;
    let p = palette_of(ui);
    let dest_ok = match t.hub.dest_mode {
        ImportDestination::ExistingStack => t.scope.is_some(),
        ImportDestination::NewStack => !t.hub.new_stack_name.trim().is_empty(),
        ImportDestination::UnstackedNotes => false,
    };
    ui.add_space(6.0);
    kit::card(ui, |ui| {
        ui.strong("PDF document ready to add");
        if !dest_ok {
            let msg = if t.hub.dest_mode == ImportDestination::UnstackedNotes {
                "PDF files must be placed into an existing or new stack."
            } else {
                "Choose an existing stack or enter a new stack name above."
            };
            ui.label(RichText::new(msg).color(p.warning));
            ui.add_enabled(
                false,
                egui::Button::new("Import PDF").min_size(vec2(130.0, 32.0)),
            );
        } else if !t.busy && kit::primary_button(ui, "Import PDF").clicked() {
            send(out, TransferMsg::Hub(Box::new(HubMsg::ImportPdf)));
        } else if t.busy {
            ui.add_enabled(
                false,
                egui::Button::new("Import PDF").min_size(vec2(130.0, 32.0)),
            );
        }
    });
}

/// The import tab content.
pub fn import_tab(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let t = &state.transfer;
    let detected = detect_format(&t.path, t.hub.is_folder, &state.extensions.importers);
    let is_pdf = is_pdf_format(detected.as_ref());

    drop_zone(ui, out);
    super::modal_found::found(state, ui, out);
    super::modal_formats::formats(ui, out);

    if let Some(text) = text_field(
        ui,
        "File or folder to import",
        &t.path,
        "/path/to/file or folder",
        1,
    ) {
        send(out, TransferMsg::Path(text));
    }

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        if ui
            .add(egui::Button::new("Choose file…").min_size(vec2(110.0, 28.0)))
            .clicked()
        {
            send(out, TransferMsg::Browse(true, false));
        }
        ui.add_space(4.0);
        if ui
            .add(egui::Button::new("Choose folder…").min_size(vec2(120.0, 28.0)))
            .clicked()
        {
            send(out, TransferMsg::Browse(true, true));
        }
        ui.add_space(8.0);
        let can_preview = !t.busy && !t.path.trim().is_empty() && !is_pdf;
        let preview_btn = egui::Button::new(
            RichText::new("Preview")
                .strong()
                .size(crate::typo::size(ui, crate::typo::Tier::Body)),
        )
        .fill(palette_of(ui).surface_hover)
        .min_size(vec2(90.0, 28.0));
        if ui.add_enabled(can_preview, preview_btn).clicked() {
            send(out, TransferMsg::Preview);
        }
    });

    if let Some(fmt) = &detected {
        ui.add_space(4.0);
        let p = palette_of(ui);
        let color = if *fmt == DetectedFormat::Unknown {
            p.warning
        } else {
            p.accent
        };
        ui.horizontal(|ui| {
            ui.label(RichText::new("Detected format:").strong());
            kit::pill(ui, &format_label(fmt), color);
        });
    }

    ui.add_space(6.0);
    super::modal_import_dest::draw(state, ui, out);

    if is_pdf {
        pdf_commit_bar(state, ui, out);
    } else {
        super::modal_transfer_preview::preview(state, ui, out);
    }
    super::modal_import_cards::draw(state, ui, out);
}
