//! The items of the selected stack (or the inbox), with a title filter and the new-card form.

use eframe::egui::{self, RichText};
use studyfrog_core::civil::format_date;

use crate::deadline_ui::deadline_of;
use crate::msg::{DraftField, Msg};
use crate::planner::PlanMsg;
use crate::state::{Selection, State};
use crate::theme::palette_of;
use crate::view::widgets::text_field;

/// The page's heading: the stack's name, "Inbox" or "All items".
fn heading(state: &State) -> String {
    match &state.browser.selection {
        Selection::Stack(folder) => state
            .vault
            .as_ref()
            .and_then(|v| v.overview.stacks.iter().find(|s| s.folder == *folder))
            .map_or_else(|| folder.clone(), |s| s.title.clone()),
        Selection::Inbox => "Inbox".to_owned(),
        Selection::Nothing => "All items".to_owned(),
    }
}

/// The search field, the count, the column menu and the table of items.
fn list(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if let Some(text) = text_field(
        ui,
        "Search titles",
        &state.browser.search,
        "type to filter",
        1,
    ) {
        out.push(Msg::SearchEdited(text));
    }
    ui.horizontal(|ui| {
        ui.label(crate::view::library_table::count_line(state));
        crate::view::library_table::column_menu(state, ui, out);
    });
    ui.allocate_ui(
        egui::vec2(
            ui.available_width(),
            (ui.available_height() * 0.6).max(260.0),
        ),
        |ui| {
            crate::view::library_table::draw(state, ui, out);
        },
    );
}

/// Heading and the list of items; for a stack also its due date, status roll-up, PDFs and the new-card form.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.label(RichText::new(heading(state)).size(26.0).strong());
            let stack = matches!(state.browser.selection, Selection::Stack(_));
            if stack {
                due_row(state, ui, out);
                crate::view::status_ui::summary(ui, &state.browser.status_counts);
                child_stacks(state, ui, out);
            }
            list(state, ui, out);
            if let Selection::Stack(folder) = &state.browser.selection {
                crate::view::library_pdfs::stack_section(state, folder, ui, out);
                card_form(state, ui, out);
            }
        });
}

fn card_form(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let (b, d) = (&state.browser, &state.browser.draft);
    ui.separator();
    ui.heading("New card");
    card_types(state, ui, out);
    let hint = b
        .card_type
        .as_deref()
        .and_then(|id| state.extensions.card_types.iter().find(|t| t.0 == id))
        .map(|t| t.2.clone());
    let fields: Vec<(DraftField, &str, &String, usize)> = match &hint {
        Some(_) => vec![(DraftField::Front, "Source text", &d.front, 4)],
        None => vec![
            (DraftField::Title, "Card title (optional)", &d.title, 1),
            (DraftField::Front, "Card front", &d.front, 3),
            (DraftField::Back, "Card back", &d.back, 3),
        ],
    };
    if let Some(h) = &hint {
        ui.weak(h);
    }
    for (field, label, current, rows) in fields {
        if let Some(text) = text_field(ui, label, current, "", rows) {
            out.push(Msg::DraftEdited(field, text));
        }
    }
    if ui
        .button(if hint.is_some() {
            "Add cards"
        } else {
            "Add card"
        })
        .clicked()
    {
        out.push(Msg::AddCardClicked);
    }
    import_panel(state, ui, out);
}

/// "Basic" and every registered card type, as a row of choices (only when extensions added some).
fn card_types(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if state.extensions.card_types.is_empty() {
        return;
    }
    let chosen = &state.browser.card_type;
    ui.horizontal_wrapped(|ui| {
        ui.label("Card type:");
        if ui
            .selectable_label(chosen.is_none(), "Basic (front and back)")
            .clicked()
        {
            out.push(Msg::CardTypeChosen(None));
        }
        for (id, name, _) in &state.extensions.card_types {
            if ui
                .selectable_label(chosen.as_deref() == Some(id), name)
                .clicked()
            {
                out.push(Msg::CardTypeChosen(Some(id.clone())));
            }
        }
    });
}

/// Import cards from a file with one of the registered importers.
fn import_panel(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if state.extensions.importers.is_empty() {
        return;
    }
    ui.separator();
    ui.heading("Import from a file");
    let kinds: Vec<String> = state
        .extensions
        .importers
        .iter()
        .map(|i| i.1.clone())
        .collect();
    ui.weak(format!("Reads: {}", kinds.join("; ")));
    if let Some(text) = text_field(
        ui,
        "File to import",
        &state.browser.import_path,
        "/path/to/file.tsv",
        1,
    ) {
        out.push(Msg::ImportPathEdited(text));
    }
    ui.horizontal(|ui| {
        if ui.button("Browse…").clicked() {
            out.push(Msg::BrowseImportClicked);
        }
        if ui.button("Import cards").clicked() {
            out.push(Msg::ImportClicked);
        }
    });
}

/// The stack's due date and the button to set or change it.
fn due_row(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let (Selection::Stack(open), Some(vault)) = (&state.browser.selection, &state.vault) else {
        return;
    };
    let Some(stack) = vault.overview.stacks.iter().find(|s| s.folder == *open) else {
        return;
    };
    let deadline = deadline_of(&state.planner.sessions, stack.uid);
    let weak = palette_of(ui).text_weak;
    ui.horizontal(|ui| {
        let (text, button) = match deadline {
            Some(s) => (
                format!("Due {}", format_date(s.start.date)),
                "Change due date",
            ),
            None => ("No due date".to_owned(), "Set due date"),
        };
        ui.label(RichText::new(text).color(weak));
        if ui.small_button(button).clicked() {
            out.push(Msg::Plan(PlanMsg::StackDeadline(stack.uid)));
        }
        if ui.small_button("Edit stack").clicked() {
            out.push(Msg::OpenItem(stack.uid));
        }
    });
}

/// Buttons for the stacks inside the open stack.
fn child_stacks(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let (Selection::Stack(open), Some(vault)) = (&state.browser.selection, &state.vault) else {
        return;
    };
    let children: Vec<_> = vault
        .overview
        .stacks
        .iter()
        .filter(|s| s.parent.as_deref() == Some(open.as_str()))
        .collect();
    if children.is_empty() {
        return;
    }
    ui.horizontal_wrapped(|ui| {
        ui.weak("Stacks inside:");
        for c in children {
            if ui.button(&c.title).clicked() {
                out.push(Msg::Select(Selection::Stack(c.folder.clone())));
            }
        }
    });
}
