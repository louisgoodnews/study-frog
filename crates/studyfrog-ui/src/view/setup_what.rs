//! Step 1 of the run setup: which stacks, kinds and tags.

use eframe::egui;
use studyfrog_core::plan::ItemKind;

use crate::msg::Msg;
use crate::setup::SetupEdit;
use crate::state::State;
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

/// Stacks as rows, then the kinds, then the tags.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let form = &state.rehearse.form;
    if let Some(vault) = &state.vault {
        let stacks = &vault.overview.stacks;
        for s in stacks {
            let nested: Vec<_> = stacks
                .iter()
                .filter(|x| inside(&x.folder, &s.folder))
                .collect();
            let mut on = form.stacks.contains(&s.uid);
            let items = s.notes + s.cards + s.questions;
            let label = if nested.is_empty() {
                format!("Stack: {} ({items} items)", s.title)
            } else {
                format!(
                    "Stack: {} ({items} items, and the {} inside it)",
                    s.title,
                    nested.len()
                )
            };
            if ui.checkbox(&mut on, label).changed() {
                let mut uids = vec![s.uid];
                uids.extend(nested.iter().map(|x| x.uid));
                edit(out, SetupEdit::Stacks(uids, on));
            }
        }
        if stacks.is_empty() {
            ui.label("There are no stacks yet. Use Create to make one.");
        }
    }
    ui.add_space(6.0);
    ui.label("Kinds of item");
    ui.horizontal(|ui| {
        for (kind, label) in KINDS {
            let mut on = form.kinds.contains(&kind);
            if ui.checkbox(&mut on, label).changed() {
                edit(out, SetupEdit::Kind(kind, on));
            }
        }
    });
    ui.weak("Nothing ticked means all kinds.");
    if let Some(text) = text_field(
        ui,
        "Tags (any of, comma separated)",
        &form.tags,
        "e.g. grammar, verbs",
        1,
    ) {
        edit(out, SetupEdit::Tags(text));
    }
}
