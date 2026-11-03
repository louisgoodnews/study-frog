//! Step 1 of the run setup: which stacks, kinds and tags.

use eframe::egui::{self, RichText};
use studyfrog_core::plan::ItemKind;

use crate::msg::Msg;
use crate::setup::SetupEdit;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::kit;
use crate::view::widgets::text_field;

const KINDS: [(ItemKind, &str); 3] = [
    (ItemKind::Note, "Notes"),
    (ItemKind::Card, "Cards"),
    (ItemKind::Question, "Questions"),
];

fn edit(out: &mut Vec<Msg>, e: SetupEdit) {
    out.push(Msg::Setup(e));
}

/// Whether `folder` is somewhere inside `outer`.
pub fn inside(folder: &str, outer: &str) -> bool {
    folder.starts_with(&format!("{outer}/")) && folder != outer
}

/// Stacks in a card, then the kinds and the tags in another.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let form = &state.rehearse.form;
    kit::group(ui, "Stacks", |ui| {
        let Some(vault) = &state.vault else { return };
        let stacks = &vault.overview.stacks;
        for s in stacks {
            let nested: Vec<_> = stacks
                .iter()
                .filter(|x| inside(&x.folder, &s.folder))
                .collect();
            let mut on = form.stacks.contains(&s.uid);
            let items = s.notes + s.cards + s.questions;
            ui.horizontal(|ui| {
                ui.add_space(18.0 * super::library::depth(stacks, s) as f32);
                if ui.checkbox(&mut on, &s.title).changed() {
                    let mut uids = vec![s.uid];
                    uids.extend(nested.iter().map(|x| x.uid));
                    edit(out, SetupEdit::Stacks(uids, on));
                }
                let weak = palette_of(ui).text_weak;
                let inside = if nested.is_empty() {
                    String::new()
                } else {
                    format!(", {} inside", nested.len())
                };
                ui.label(RichText::new(format!("{items} items{inside}")).color(weak));
            });
        }
        if stacks.is_empty() {
            ui.label("There are no stacks yet. Use Create to make one.");
        }
    });
    kit::group(ui, "Narrow it down (optional)", |ui| {
        ui.label("Kinds of item");
        ui.horizontal(|ui| {
            for (kind, label) in KINDS {
                let mut on = form.kinds.contains(&kind);
                if ui.checkbox(&mut on, label).changed() {
                    edit(out, SetupEdit::Kind(kind, on));
                }
            }
            ui.weak("Nothing ticked means all kinds.");
        });
        ui.add_space(6.0);
        if let Some(text) = text_field(
            ui,
            "Tags (any of, comma separated)",
            &form.tags,
            "e.g. grammar, verbs",
            1,
        ) {
            edit(out, SetupEdit::Tags(text));
        }
    });
}
