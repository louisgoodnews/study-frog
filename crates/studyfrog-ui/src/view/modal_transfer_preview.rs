//! The preview half of the import dialog: counts, warnings, item preview, duplicate policies, and the Import action.

use eframe::egui::{self, RichText, vec2};
use studyfrog_service::xfer_plan::{Policy, Verdict};

use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;
use crate::transfer_ui::TransferMsg;
use crate::view::kit;

fn send(out: &mut Vec<Msg>, m: TransferMsg) {
    out.push(Msg::Library(crate::library_ui::LibraryMsg::Transfer(m)));
}

fn verdict_text(v: &Verdict) -> String {
    match v {
        Verdict::Create => "new".to_owned(),
        Verdict::Duplicate(p) => format!("exists at {p}"),
        Verdict::Error(e) => format!("error: {e}"),
    }
}

fn verdict_badge(ui: &mut egui::Ui, v: &Verdict) {
    let p = palette_of(ui);
    let (text, color) = match v {
        Verdict::Create => ("new", p.success),
        Verdict::Duplicate(_) => ("exists", p.warning),
        Verdict::Error(_) => ("error", p.danger),
    };
    kit::pill(ui, text, color);
}

fn line_item(ui: &mut egui::Ui, title: &str, folder: &str, v: &Verdict) {
    ui.horizontal(|ui| {
        verdict_badge(ui, v);
        ui.add_space(2.0);
        ui.label(RichText::new(title).strong());
        ui.weak(format!("in {folder} ({})", verdict_text(v)));
    });
}

fn duplicate_policy(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let t = &state.transfer;
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new("Items that exist:").strong());
        for (pol, label) in [
            (Policy::Skip, "Skip"),
            (Policy::Replace, "Replace"),
            (Policy::KeepBoth, "Keep both"),
        ] {
            if ui.selectable_label(t.policy == pol, label).clicked() {
                send(out, TransferMsg::Policy(pol));
            }
        }
    });
}

/// The preview section when a file or folder is analyzed.
pub fn preview(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let t = &state.transfer;
    let Some(p) = &t.preview else { return };
    let (create, dup, err) = p.counts;

    ui.add_space(6.0);
    ui.separator();
    ui.add_space(4.0);

    ui.strong(format!("{} found {} item(s)", p.adapter, p.total));
    ui.label(format!(
        "{create} to create, {dup} already exist, {err} with errors, {} new stack(s), {} picture or PDF file(s)",
        p.new_stacks, p.files
    ));

    if create > 0 || dup > 0 || err > 0 {
        ui.add_space(2.0);
        let pal = palette_of(ui);
        ui.horizontal_wrapped(|ui| {
            if create > 0 {
                kit::pill(ui, &format!("{create} new"), pal.success);
            }
            if dup > 0 {
                kit::pill(ui, &format!("{dup} duplicate(s)"), pal.warning);
            }
            if err > 0 {
                kit::pill(ui, &format!("{err} error(s)"), pal.danger);
            }
            if p.new_stacks > 0 {
                kit::pill(ui, &format!("{} new stack(s)", p.new_stacks), pal.warning);
            }
        });
        if p.new_stacks > 0 && t.scope.is_some() && p.csv.is_some() {
            let target = t.scope.as_deref().unwrap_or_default();
            ui.add_space(2.0);
            if ui
                .button(format!("Put all items directly into '{target}'"))
                .on_hover_text(
                    "Ignore sub-stack paths in the file so items land directly in the chosen stack",
                )
                .clicked()
            {
                send(
                    out,
                    TransferMsg::Csv(crate::transfer_csv::CsvChange::Column(
                        crate::transfer_csv::Role::Stack,
                        None,
                    )),
                );
            }
        }
    }

    for w in &p.warnings {
        ui.weak(w);
    }

    ui.add_space(4.0);
    egui::ScrollArea::vertical()
        .id_salt("preview-items-scroll")
        .max_height(140.0)
        .show(ui, |ui| {
            for l in &p.lines {
                line_item(ui, &l.title, &l.folder, &l.verdict);
            }
        });

    if dup > 0 {
        duplicate_policy(state, ui, out);
    }

    if let Some(info) = &p.csv {
        ui.add_space(4.0);
        super::modal_csv::settings(info, state.prompt_run.reply.is_some(), ui, out);
    }

    if p.scheduling {
        ui.add_space(2.0);
        let mut on = t.with_schedule;
        if ui.checkbox(&mut on, "Import the review history").changed() {
            send(out, TransferMsg::WithSchedule(on));
        }
    }

    ui.add_space(6.0);
    let can_import = !t.busy && create + dup > 0;
    let import_btn = egui::Button::new(
        RichText::new("Import")
            .strong()
            .size(crate::typo::size(ui, crate::typo::Tier::Body))
            .color(palette_of(ui).on_accent),
    )
    .fill(palette_of(ui).accent)
    .min_size(vec2(130.0, 32.0));

    if ui.add_enabled(can_import, import_btn).clicked() {
        send(out, TransferMsg::Import);
    }
}
