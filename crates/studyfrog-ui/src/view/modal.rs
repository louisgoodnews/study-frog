//! The dialogs: create an item, and choose the vault. Modelled on a workspace app's "create" dialog:
//! kind choice and close button on top, a big title, the text fields, the stack it goes into, and a
//! footer with "Create another" and the primary button.

use eframe::egui::{self, Align, Layout};
use studyfrog_service::create_more::NewKind;

use crate::image_ui::ImageTarget;
use crate::modal::{CreateForm, Field, Modal, ModalMsg};
use crate::msg::Msg;
use crate::state::State;
use crate::view::icons::{Glyph, icon_button};
use crate::view::kit;
use crate::view::md_editor::{Mode, md_editor};
use crate::view::widgets::text_field;

pub(super) const WIDTH: f32 = 560.0;

/// A note gets a wide dialog: the editor needs room for the text and its rendering.
const NOTE_WIDTH: f32 = 900.0;

fn send(out: &mut Vec<Msg>, m: ModalMsg) {
    out.push(Msg::Modal(m));
}

/// The open dialog, over a dimmed window; Escape or a click outside closes it.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(modal) = &state.modal else { return };
    let response = egui::Modal::new(egui::Id::unique("dialog")).show(ui.ctx(), |ui| {
        ui.set_width(match modal {
            Modal::Create(f) if f.kind == NewKind::Note => NOTE_WIDTH,
            _ => WIDTH,
        });
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
        if ui.button("Add PDF ...").clicked() {
            let folder = f.parent.clone().unwrap_or_default();
            out.push(Msg::Pdf(crate::pdf_types::PdfMsg::Library(
                crate::pdf_library_types::LibraryMsg::AddTo(folder),
            )));
            send(out, ModalMsg::Close);
        }
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
    let (rows, mode) = match f.kind {
        NewKind::Note => (14, Mode::Split),
        _ => (4, Mode::Write),
    };
    ui.weak(text_hint);
    if let Some(t) = md_editor(
        ui,
        text_label,
        &f.text,
        (rows, mode),
        Some(ImageTarget::Create(Field::Text)),
        out,
    ) {
        send(out, ModalMsg::Edit(Field::Text, t));
    }
    let answer = answer_label.and_then(|label| {
        md_editor(
            ui,
            label,
            &f.answer,
            (3, Mode::Write),
            Some(ImageTarget::Create(Field::Answer)),
            out,
        )
    });
    if let Some(t) = answer {
        send(out, ModalMsg::Edit(Field::Answer, t));
    }
    super::modal_fields::parent_picker(state, f, ui, out);
    if f.kind == NewKind::Stack {
        super::modal_fields::due_fields(state, f, ui, out);
    }
    if let Some(t) = text_field(ui, "Tags", &f.tags, "comma separated", 1) {
        send(out, ModalMsg::Edit(Field::Tags, t));
    }
    if let Some(e) = &f.error {
        ui.colored_label(ui.visuals().error_fg_color, e);
    }
    footer(f, ui, out);
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
            let create = ui
                .add_enabled_ui(!f.busy, |ui| kit::primary_button(ui, label))
                .inner;
            if create.clicked() {
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
