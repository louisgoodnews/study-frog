//! The CSV settings under the preview: delimiter, encoding, header, what each column means, the first rows and the
//! problems of the file, all at once.

use eframe::egui::{self, ComboBox};
use studyfrog_service::xfer_adapters::CSV_LAYOUT;
use studyfrog_service::xfer_csv_map::{ai_message, problem_text};

use crate::library_ui::LibraryMsg;
use crate::msg::Msg;
use crate::transfer_csv::{CsvChange, Role};
use crate::transfer_ui::{CsvInfo, TransferMsg};

const DELIMITERS: [(char, &str); 4] = [
    (',', "Comma"),
    (';', "Semicolon"),
    ('\t', "Tab"),
    ('|', "Bar"),
];
const SHOWN_PROBLEMS: usize = 50;

fn send(out: &mut Vec<Msg>, c: CsvChange) {
    out.push(Msg::Library(LibraryMsg::Transfer(TransferMsg::Csv(c))));
}

fn delimiter_name(d: char) -> &'static str {
    DELIMITERS
        .iter()
        .find(|(c, _)| *c == d)
        .map_or("Other", |(_, n)| n)
}

fn column_menu(info: &CsvInfo, ui: &mut egui::Ui, (role, label): (Role, &str), out: &mut Vec<Msg>) {
    let m = &info.sheet.map;
    let current = match role {
        Role::Front => Some(m.front),
        Role::Back => Some(m.back),
        Role::Title => m.title,
        Role::Tags => m.tags,
        Role::Stack => m.stack,
        Role::Kind => m.kind,
    };
    let optional = !matches!(role, Role::Front | Role::Back);
    let shown = current
        .and_then(|i| info.sheet.names.get(i))
        .map_or("(none)", String::as_str);
    ComboBox::from_label(label)
        .selected_text(shown)
        .show_ui(ui, |ui| {
            if optional && ui.selectable_label(current.is_none(), "(none)").clicked() {
                send(out, CsvChange::Column(role, None));
            }
            for (i, name) in info.sheet.names.iter().enumerate() {
                if ui.selectable_label(current == Some(i), name).clicked() {
                    send(out, CsvChange::Column(role, Some(i)));
                }
            }
        });
}

fn choices(info: &CsvInfo, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let s = &info.sheet;
    ComboBox::from_label("Delimiter")
        .selected_text(delimiter_name(s.delimiter))
        .show_ui(ui, |ui| {
            for (c, name) in DELIMITERS {
                if ui.selectable_label(s.delimiter == c, name).clicked() {
                    send(out, CsvChange::Delimiter(Some(c)));
                }
            }
        });
    let encoding = if s.latin1 { "Latin-1" } else { "UTF-8" };
    ComboBox::from_label("Text encoding")
        .selected_text(encoding)
        .show_ui(ui, |ui| {
            for (latin1, name) in [(false, "UTF-8"), (true, "Latin-1")] {
                if ui.selectable_label(s.latin1 == latin1, name).clicked() {
                    send(out, CsvChange::Latin1(Some(latin1)));
                }
            }
        });
    let mut header = s.header;
    if ui
        .checkbox(&mut header, "The first row names the columns")
        .changed()
    {
        send(out, CsvChange::Header(Some(header)));
    }
    for pair in [
        (Role::Front, "Front"),
        (Role::Back, "Back"),
        (Role::Title, "Title"),
        (Role::Tags, "Tags"),
        (Role::Stack, "Stack"),
        (Role::Kind, "Kind"),
    ] {
        column_menu(info, ui, pair, out);
    }
}

fn rows(info: &CsvInfo, ui: &mut egui::Ui) {
    egui::ScrollArea::horizontal()
        .id_salt("csv-rows")
        .show(ui, |ui| {
            egui::Grid::new("csv-first-rows")
                .striped(true)
                .show(ui, |ui| {
                    for name in &info.sheet.names {
                        ui.strong(name);
                    }
                    ui.end_row();
                    for row in &info.sheet.first_rows {
                        for i in 0..info.sheet.names.len() {
                            let cell = row.get(i).map_or("", String::as_str);
                            ui.label(if cell.is_empty() { "–" } else { cell });
                        }
                        ui.end_row();
                    }
                });
        });
}

fn problems(info: &CsvInfo, ui: &mut egui::Ui) {
    let n = info.problems.len();
    ui.colored_label(
        ui.visuals().error_fg_color,
        format!("{n} problem(s); those rows are left out:"),
    );
    egui::ScrollArea::vertical()
        .id_salt("csv-problems")
        .max_height(110.0)
        .show(ui, |ui| {
            for p in info.problems.iter().take(SHOWN_PROBLEMS) {
                ui.label(problem_text(p));
            }
            if n > SHOWN_PROBLEMS {
                ui.weak(format!(
                    "… and {} more (all are in the message for the AI)",
                    n - SHOWN_PROBLEMS
                ));
            }
        });
    if ui.button("Copy the problems for the AI").clicked() {
        ui.ctx().copy_text(ai_message(CSV_LAYOUT, &info.problems));
    }
}

/// The settings, the first rows and the problems of a CSV or TSV file.
pub fn settings(info: &CsvInfo, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    egui::CollapsingHeader::new("How the file is read (CSV)")
        .id_salt("csv-settings")
        .default_open(true)
        .show(ui, |ui| {
            choices(info, ui, out);
            rows(info, ui);
            if !info.problems.is_empty() {
                problems(info, ui);
            }
        });
}
