//! The console's output controls (STORY-0152): the stream filters, a search over the text, the export to the vault's
//! log file, and the rule that decides which entries are shown.

use eframe::egui::{self, RichText};

use crate::console_ui::{ConsoleMsg, ConsoleState, Entry, EntryKind};
use crate::msg::Msg;
use crate::script_studio::ConsoleFilter;
use crate::state::State;
use crate::view::kit;

fn is_visible(e: &Entry, filter: ConsoleFilter) -> bool {
    match filter {
        ConsoleFilter::All => true,
        ConsoleFilter::Stdout => matches!(
            e.kind,
            EntryKind::Command | EntryKind::Answer | EntryKind::Script | EntryKind::Stdout
        ),
        ConsoleFilter::Stderr => matches!(e.kind, EntryKind::Error | EntryKind::Stderr),
        ConsoleFilter::Events => matches!(e.kind, EntryKind::Event),
    }
}

/// Whether an entry is shown: its stream is the one chosen, and its text matches the search.
pub fn visible(e: &Entry, c: &ConsoleState) -> bool {
    is_visible(e, c.studio.filter) && crate::console_ui::lines::matches(e, &c.search)
}

/// The stream filters, the search box, "Export log" and "Clear Output".
pub fn bar(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let active = state.console.studio.filter;
    // Wraps on a narrow window, so the export and clear buttons never leave the screen.
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("Streams:").strong());
        let filters = [
            (ConsoleFilter::All, "All"),
            (ConsoleFilter::Stdout, "Stdout"),
            (ConsoleFilter::Stderr, "Stderr / Errors"),
            (ConsoleFilter::Events, "Events"),
        ];
        for (f, name) in filters {
            if ui.selectable_label(active == f, name).clicked() {
                out.push(Msg::Console(ConsoleMsg::StudioSetFilter(f)));
            }
        }
        let mut query = state.console.search.clone();
        if ui
            .add(egui::TextEdit::singleline(&mut query).hint_text("Search output").desired_width(140.0))
            .changed()
        {
            out.push(Msg::Console(ConsoleMsg::Search(query)));
        }
            if kit::quiet_button(ui, "Export log")
            .on_hover_text("Writes the whole transcript, with the time of each line, to logs/script_runs.log in the vault.")
            .clicked()
        {
            out.push(Msg::Console(ConsoleMsg::ExportLog));
        }
        if kit::quiet_button(ui, "Jobs")
            .on_hover_text("The scheduled and event-bound jobs: their next runs, switching them on or off, and running one now.")
            .clicked()
        {
            out.push(Msg::Console(ConsoleMsg::OpenJobs));
        }
        if kit::quiet_button(ui, "Clear Output")
            .on_hover_text("Clears the output of the script.")
            .clicked()
        {
            out.push(Msg::Console(ConsoleMsg::Clear));
        }
    });
}
