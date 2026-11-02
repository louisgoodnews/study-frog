//! The Goals tab: the stack's goals as a Markdown bullet list, edited in place and saved to the stack's file.

use eframe::egui;

use crate::library_ui::LibraryMsg;
use crate::msg::Msg;
use crate::stack_view_ui::StackViewMsg;
use crate::state::State;
use crate::view::md_editor::{Mode, md_editor};

fn send(out: &mut Vec<Msg>, m: StackViewMsg) {
    out.push(Msg::Library(LibraryMsg::Stack(m)));
}

/// The tab.
pub fn draw(state: &State, folder: &str, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(g) = state
        .library
        .stack_view
        .goals
        .as_ref()
        .filter(|g| g.folder == folder)
    else {
        ui.label("Reading the goals…");
        return;
    };
    ui.weak("What do you want to reach with this stack? Write a bullet list; check boxes (- [ ]) are fine. It is saved under a Goals heading in the stack's file.");
    if let Some(text) = md_editor(ui, "Goals", &g.draft, (8, Mode::Write), None, out) {
        send(out, StackViewMsg::GoalsEdited(text));
    }
    let dirty = g.draft != g.saved;
    ui.horizontal(|ui| {
        if ui
            .add_enabled(dirty && !g.saving, egui::Button::new("Save goals"))
            .clicked()
        {
            send(out, StackViewMsg::GoalsSave);
        }
        if dirty && ui.button("Discard changes").clicked() {
            send(out, StackViewMsg::GoalsEdited(g.saved.clone()));
        }
        if dirty {
            ui.weak("Unsaved changes");
        }
    });
}
