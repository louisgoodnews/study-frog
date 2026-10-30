//! The dialogs: create an item, and choose the vault. Modelled on a workspace app's "create" dialog:
//! kind choice and close button on top, a big title, the text fields, the stack it goes into, and a
//! footer with "Create another" and the primary button.

use eframe::egui::{self, Align, Color32, ComboBox, Layout, RichText, Role, WidgetInfo};
use studyfrog_service::create_more::NewKind;

use crate::modal::{CreateForm, Field, Modal, ModalMsg, stack_labels};
use crate::msg::Msg;
use crate::state::State;
use crate::view::icons::{Glyph, icon_button};
use crate::view::widgets::text_field;

const WIDTH: f32 = 560.0;

fn send(out: &mut Vec<Msg>, m: ModalMsg) {
    out.push(Msg::Modal(m));
}

/// The open dialog, over a dimmed window; Escape or a click outside closes it.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(modal) = &state.modal else { return };
    let response = egui::Modal::new(egui::Id::unique("dialog")).show(ui.ctx(), |ui| {
        ui.set_width(WIDTH);
        match modal {
            Modal::Create(form) => create(state, form, ui, out),
            Modal::Vault => vault(state, ui, out),
        }
    });
    if response.should_close() {
        send(out, ModalMsg::Close);
    }
}

fn header(ui: &mut egui::Ui, out: &mut Vec<Msg>, add: impl FnOnce(&mut egui::Ui, &mut Vec<Msg>)) {
    ui.horizontal(|ui| {
        add(ui, out);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if icon_button(ui, Glyph::Close, "Close", false).clicked() {
                send(out, ModalMsg::Close);
            }
        });
    });
}

const KINDS: [(NewKind, &str); 4] = [
    (NewKind::Stack, "Stack"),
    (NewKind::Card, "Card"),
    (NewKind::Note, "Note"),
    (NewKind::Question, "Question"),
];

/// (field label, hint) of the main text, and the label of the second text if the kind has one.
fn labels(kind: NewKind) -> ((&'static str, &'static str), Option<&'static str>) {
    match kind {
        NewKind::Stack => (("Description", "What is in this stack (optional)"), None),
        NewKind::Note => (("Body", "Markdown is supported"), None),
        NewKind::Card => (("Front", "The question or prompt"), Some("Back")),
        NewKind::Question => (
            ("Prompt", "The question to answer"),
            Some("Reference answer"),
        ),
    }
}

fn create(state: &State, f: &CreateForm, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    header(ui, out, |ui, out| {
        ui.strong("New item");
        for (kind, label) in KINDS {
            if ui.selectable_label(f.kind == kind, label).clicked() {
                send(out, ModalMsg::SetKind(kind));
            }
        }
    });
    ui.add_space(6.0);
    let title_hint = match f.kind {
        NewKind::Card | NewKind::Question => "Title (optional: taken from the text)",
        _ => "Title",
    };
    if let Some(t) = text_field(ui, "Title", &f.title, title_hint, 1) {
        send(out, ModalMsg::Edit(Field::Title, t));
    }
    let ((text_label, text_hint), answer_label) = labels(f.kind);
    if let Some(t) = text_field(ui, text_label, &f.text, text_hint, 4) {
        send(out, ModalMsg::Edit(Field::Text, t));
    }
    if let Some(t) = answer_label.and_then(|label| text_field(ui, label, &f.answer, "", 3)) {
        send(out, ModalMsg::Edit(Field::Answer, t));
    }
    parent_picker(state, f, ui, out);
    if let Some(t) = text_field(ui, "Tags", &f.tags, "comma separated", 1) {
        send(out, ModalMsg::Edit(Field::Tags, t));
    }
    if let Some(e) = &f.error {
        ui.colored_label(ui.visuals().error_fg_color, e);
    }
    footer(f, ui, out);
}

fn parent_picker(state: &State, f: &CreateForm, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let stacks = state
        .vault
        .as_ref()
        .map(|v| stack_labels(&v.overview))
        .unwrap_or_default();
    let is_stack = f.kind == NewKind::Stack;
    if stacks.is_empty() && !is_stack {
        ui.colored_label(Color32::from_rgb(230, 170, 60), "There is no stack yet. Every card, note and question lives in a stack: create a stack first.");
        return;
    }
    let none = "None (top level)";
    let current = f
        .parent
        .as_ref()
        .and_then(|p| stacks.iter().find(|(folder, _)| folder == p))
        .map_or(none.to_owned(), |(_, label)| label.clone());
    let label = if is_stack {
        "Inside stack (optional)"
    } else {
        "Parent stack"
    };
    ui.label(label);
    let picker = ComboBox::from_id_salt("parent-stack")
        .selected_text(current)
        .width(WIDTH - 24.0)
        .show_ui(ui, |ui| {
            if is_stack && ui.selectable_label(f.parent.is_none(), none).clicked() {
                send(out, ModalMsg::SetParent(None));
            }
            for (folder, label) in &stacks {
                if ui
                    .selectable_label(f.parent.as_deref() == Some(folder), label)
                    .clicked()
                {
                    send(out, ModalMsg::SetParent(Some(folder.clone())));
                }
            }
        });
    picker
        .response
        .widget_info(|| WidgetInfo::labeled(Role::ComboBox, true, label));
}

fn footer(f: &CreateForm, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.separator();
    ui.horizontal(|ui| {
        let mut another = f.another;
        if ui.checkbox(&mut another, "Create another").changed() {
            send(out, ModalMsg::SetAnother(another));
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let label = if f.busy { "Creating…" } else { "Create" };
            let button = egui::Button::new(RichText::new(label).color(Color32::BLACK))
                .fill(Color32::from_rgb(110, 160, 245));
            if ui.add_enabled(!f.busy, button).clicked() {
                send(out, ModalMsg::Submit);
            }
            if ui.button("Cancel").clicked() {
                send(out, ModalMsg::Close);
            }
        });
    });
}

fn vault(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    header(ui, out, |ui, _| {
        ui.strong("Open or create a vault");
    });
    ui.add_space(6.0);
    ui.label("A vault is a folder of Markdown files that StudyFrog reads and writes. Choose an existing folder, or an empty one to start a new vault.");
    if let Some(t) = text_field(
        ui,
        "Vault folder",
        &state.path_input,
        "/path/to/your/vault",
        1,
    ) {
        out.push(Msg::PathEdited(t));
    }
    ui.horizontal(|ui| {
        if ui
            .add_enabled(!state.opening, egui::Button::new("Choose folder…"))
            .clicked()
        {
            out.push(Msg::BrowseVaultClicked);
        }
        if ui
            .add_enabled(!state.opening, egui::Button::new("Open vault"))
            .clicked()
        {
            out.push(Msg::OpenVaultClicked);
        }
        if state.opening {
            ui.spinner();
        }
    });
    if !state.app.recent.is_empty() {
        ui.separator();
        ui.strong("Recent vaults");
        for path in &state.app.recent {
            if ui
                .add_enabled(!state.opening, egui::Button::new(path.as_str()))
                .clicked()
            {
                out.push(Msg::OpenRecent(path.clone()));
            }
        }
    }
}
