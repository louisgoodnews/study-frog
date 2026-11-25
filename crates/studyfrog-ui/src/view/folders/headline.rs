//! The Folders headline (EPIC-073, STORY-0233): the title and the counts on the gradient panel.

use eframe::egui::{self, RichText};
use studyfrog_core::count::counted;

use crate::folders_tree::entries_from_vault;
use crate::folders_types::FolderEntry;
use crate::state::State;
use crate::view::gradient_panel;

/// The height of the headline panel, in points.
const HEADLINE_HEIGHT: f32 = 64.0;

/// The title and the folder and file counts, on the gradient panel.
pub fn draw(state: &State, ui: &mut egui::Ui) {
    let (folders, files) = counts(state);
    gradient_panel::panel(ui, HEADLINE_HEIGHT, |ui, s| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Folders")
                    .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                    .strong()
                    .color(s.gradient_text),
            );
            let text = format!(
                "{}, {}",
                counted(folders, "folder", "folders"),
                counted(files, "file", "files")
            );
            ui.label(RichText::new(text).color(s.gradient_text));
        });
    });
}

/// The counts of folders and of other entries, from the same source the tree draws from.
fn counts(state: &State) -> (usize, usize) {
    if state.folders.entries.is_empty()
        && let Some(v) = &state.vault
    {
        return tally(&entries_from_vault(v, &state.library.items));
    }
    tally(&state.folders.entries)
}

fn tally(entries: &[FolderEntry]) -> (usize, usize) {
    let folders = entries.iter().filter(|e| e.is_folder).count();
    (folders, entries.len() - folders)
}
