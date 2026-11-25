//! The console's output controls (STORY-0152): the stream filters, a search over the text, the export to the vault's
//! log file, and the rule that decides which entries are shown. The entry list (STORY-0239) is drawn here too.

use eframe::egui::{self, RichText, TextStyle};

use crate::console_ui::{ConsoleMsg, ConsoleState, Entry, EntryKind};
use crate::msg::Msg;
use crate::script_studio::ConsoleFilter;
use crate::state::State;
use crate::theme::palette_of;
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
            .add(
                egui::TextEdit::singleline(&mut query)
                    .hint_text("Search output")
                    .desired_width(140.0),
            )
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

/// The entries that pass the filters, newest at the bottom, in a scroll area that takes the rest of the card.
pub fn list(state: &State, ui: &mut egui::Ui) {
    let pal = palette_of(ui);
    let visible_entries: Vec<_> = state
        .console
        .entries
        .iter()
        .filter(|e| visible(e, &state.console))
        .collect();

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            if visible_entries.is_empty() {
                ui.label(
                    RichText::new(
                        "No entries for selected filter. Type a command or run FrogScript above.",
                    )
                    .color(pal.text_weak),
                );
            }
            for e in visible_entries {
                entry(ui, e);
                ui.add_space(2.0);
            }
        });
}

/// One line: its time, its stream's prefix, and its text, in the stream's colour.
fn entry(ui: &mut egui::Ui, e: &Entry) {
    let p = palette_of(ui);
    let (prefix, color) = match e.kind {
        EntryKind::Command => ("› ", p.accent),
        EntryKind::Answer => ("", p.text),
        EntryKind::Error => ("error: ", p.danger),
        EntryKind::Event => ("event ", p.text_weak),
        EntryKind::Script => ("+ ", p.success),
        EntryKind::Stdout => ("stdout: ", p.text),
        EntryKind::Stderr => ("stderr: ", p.danger),
    };
    // Each line carries its time on the app's clock, so a failure can be placed among the other lines.
    let text = RichText::new(format!("{:>8.1}s  {prefix}{}", e.at, e.text))
        .text_style(TextStyle::Monospace)
        .color(color);
    ui.add(egui::Label::new(text).wrap().selectable(true));
}
