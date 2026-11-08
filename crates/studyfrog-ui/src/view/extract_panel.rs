//! A note's "Extract / Sync cards" button and the panel that previews and runs the sync (STORY-0110).

use eframe::egui::{self, RichText};
use studyfrog_md::doc::Doc;
use studyfrog_md::inline_extract::{InlineKind, extract_inline};
use studyfrog_service::inline_sync::{SyncChange, SyncEntry, count_of};

use crate::editor::Editor;
use crate::msg::Msg;
use crate::recall_types::{ExtractPanel, RecallMsg};
use crate::state::State;
use crate::theme::palette_of;
use crate::view::kit;

fn msg(m: RecallMsg) -> Msg {
    Msg::Recall(m)
}

/// The toolbar button of a note that has inline cards: "Extract / Sync N cards…".
pub fn button(e: &Editor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if !matches!(e.doc, Doc::Note(_)) {
        return;
    }
    let Some(body) = e.form.fields.first().map(|f| &f.text) else {
        return;
    };
    let n = extract_inline(body).len();
    if n == 0 {
        return;
    }
    let label = format!("Extract / Sync {n} card(s)…");
    let tip = "Make or update the cards and questions written inline in this note";
    if ui.button(label).on_hover_text(tip).clicked() {
        out.push(msg(RecallMsg::ExtractPreview(e.uid, body.clone())));
    }
}

fn change_word(c: SyncChange) -> &'static str {
    match c {
        SyncChange::New => "new",
        SyncChange::Updated => "updated",
        SyncChange::Unchanged => "unchanged",
    }
}

fn entry_row(entry: &SyncEntry, ui: &mut egui::Ui) {
    let p = palette_of(ui);
    ui.horizontal_wrapped(|ui| {
        let strong = entry.change != SyncChange::Unchanged;
        let color = if strong { p.accent } else { p.text_weak };
        kit::pill(ui, change_word(entry.change), color);
        let kind = match entry.block.kind {
            InlineKind::Card => "Card",
            InlineKind::Question => "Question",
        };
        ui.label(RichText::new(format!("{kind}: {}", entry.block.front)).strong());
        ui.label(format!("- {}", entry.block.back));
        if let Some(h) = &entry.block.heading {
            ui.weak(format!("(under {h})"));
        }
    });
}

fn summary(panel: &ExtractPanel, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(plan) = &panel.plan else { return };
    let (new, upd) = (
        count_of(plan, SyncChange::New),
        count_of(plan, SyncChange::Updated),
    );
    let same = count_of(plan, SyncChange::Unchanged);
    ui.label(format!(
        "{new} new, {upd} updated, {same} unchanged, in the stack {}. Each links back to [[{}]].",
        plan.folder, plan.title
    ));
    for entry in &plan.entries {
        entry_row(entry, ui);
    }
    ui.add_space(4.0);
    let ready = new + upd > 0 && !panel.busy;
    let sync = ui.add_enabled_ui(ready, |ui| kit::primary_button(ui, "Sync cards"));
    if sync.inner.clicked() {
        out.push(msg(RecallMsg::ExtractSync));
    }
}

/// The panel under the toolbar while it is open for this note.
pub fn panel(state: &State, e: &Editor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let panel = &state.recall.extract;
    if panel.note != Some(e.uid) {
        return;
    }
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(RichText::new("Extract / Sync cards").strong());
            if ui.small_button("Close the extraction").clicked() {
                out.push(msg(RecallMsg::ExtractClose));
            }
        });
        if panel.busy {
            ui.weak("Working…");
        }
        if let Some(why) = &panel.error {
            ui.colored_label(ui.visuals().error_fg_color, why);
        }
        if let Some((c, u)) = panel.done {
            ui.label(format!("Done: {c} created, {u} updated."));
            let undo = crate::library_bulk::BulkMsg::Undo;
            if state.library.undo.is_some() && ui.button("Undo the sync").clicked() {
                out.push(Msg::Library(crate::library_ui::LibraryMsg::Bulk(undo)));
            }
        }
        summary(panel, ui, out);
    });
}
