//! The Create dialog, after Jira's: the stack and the type in the top left corner, a large title, the description
//! and the other text as Markdown editors, then the fields. The dialog can be widened by dragging its grip or with
//! the Enlarge button, and every editor can be made taller by dragging its lower edge.

use eframe::egui::{self, Align, ComboBox, Layout, RichText, Role, TextEdit, WidgetInfo};
use studyfrog_core::event::Difficulty;
use studyfrog_core::priority::Priority;
use studyfrog_service::create_more::NewKind;

use crate::image_ui::ImageTarget;
use crate::modal::{CreateForm, Field, ModalMsg};
use crate::msg::Msg;
use crate::state::State;
use crate::view::md_editor::{Mode, md_editor};
use crate::view::widgets::text_field;
use crate::view::{kit, modal_fields};

const KINDS: [(NewKind, &str); 4] = [
    (NewKind::Card, "Card"),
    (NewKind::Note, "Note"),
    (NewKind::Question, "Question"),
    (NewKind::Stack, "Stack"),
];

fn send(out: &mut Vec<Msg>, m: ModalMsg) {
    out.push(Msg::Modal(m));
}

fn kind_label(kind: NewKind) -> &'static str {
    KINDS
        .iter()
        .find(|(k, _)| *k == kind)
        .map_or("Card", |(_, l)| l)
}

fn type_combo(f: &CreateForm, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let r = ComboBox::from_id_salt("item-type")
        .selected_text(kind_label(f.kind))
        .width(120.0)
        .show_ui(ui, |ui| {
            for (kind, label) in KINDS {
                if ui.selectable_label(f.kind == kind, label).clicked() {
                    super::modal_size::reset(ui.ctx());
                    send(out, ModalMsg::SetKind(kind));
                }
            }
        });
    r.response
        .widget_info(|| WidgetInfo::labeled(Role::ComboBox, true, "Type of item"));
}

fn header(state: &State, f: &CreateForm, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    // The window controls have a row of their own, so the pickers below can never push them out.
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Create an item")
                .size(crate::typo::size(ui, crate::typo::Tier::Body))
                .strong(),
        );
        let ctx = ui.ctx().clone();
        super::modal_size::window_buttons(&ctx, f, ui, out);
    });
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        modal_fields::parent_picker(state, f, ui, out);
        modal_fields::also_in_picker(state, f, ui, out);
        ui.separator();
        type_combo(f, ui, out);
    });
}

/// (label, hint) of the main text, and the label of the second text if the kind has one.
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

fn title_field(f: &CreateForm, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let label = ui.label(
        RichText::new("Title")
            .size(crate::typo::size(ui, crate::typo::Tier::Label))
            .weak(),
    );
    let mut text = f.title.clone();
    let hint = match f.kind {
        NewKind::Card | NewKind::Question => "Title (optional: taken from the text)",
        _ => "Title",
    };
    let edit = TextEdit::singleline(&mut text)
        .font(egui::TextStyle::Heading)
        .hint_text(hint)
        .desired_width(f32::INFINITY);
    ui.add(edit).labelled_by(label.id);
    if text != f.title {
        send(out, ModalMsg::Edit(Field::Title, text));
    }
}

fn editors(f: &CreateForm, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if f.kind == NewKind::Stack && !f.is_stack {
        return;
    }
    let ((text_label, text_hint), answer_label) = labels(f.kind);
    let (rows, mode) = match f.kind {
        NewKind::Note => (16, Mode::Write),
        _ => (5, Mode::Write),
    };
    ui.weak(text_hint);
    let image = Some(ImageTarget::Create(Field::Text));
    if let Some(t) = md_editor(ui, text_label, &f.text, (rows, mode), image, out) {
        send(out, ModalMsg::Edit(Field::Text, t));
    }
    let Some(label) = answer_label else { return };
    let image = Some(ImageTarget::Create(Field::Answer));
    if let Some(t) = md_editor(ui, label, &f.answer, (4, Mode::Write), image, out) {
        send(out, ModalMsg::Edit(Field::Answer, t));
    }
}

fn fields(state: &State, f: &CreateForm, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if f.kind == NewKind::Card {
        let has_extra = !f.tags.is_empty()
            || f.priority != Priority::Medium
            || f.difficulty != Difficulty::Difficult
            || !f.due.is_empty();
        ui.add_space(4.0);
        egui::CollapsingHeader::new("More fields")
            .default_open(has_extra)
            .show(ui, |ui| modal_fields::metadata_fields(state, f, ui, out));
        return;
    }
    if f.kind != NewKind::Stack {
        modal_fields::metadata_fields(state, f, ui, out);
    }
    if f.kind == NewKind::Stack {
        ui.horizontal_wrapped(|ui| {
            modal_fields::priority_choice(f, ui, out);
            let mut is_stack = f.is_stack;
            if ui.checkbox(&mut is_stack, "Mark as stack folder").changed() {
                send(out, ModalMsg::SetIsStack(is_stack));
            }
        });
        if let Some(t) = text_field(ui, "Tags", &f.tags, "comma separated", 1) {
            send(out, ModalMsg::Edit(Field::Tags, t));
        }
        if f.is_stack {
            modal_fields::due_fields(state, f, ui, out);
        }
    }
}

fn footer(f: &CreateForm, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.separator();
    ui.horizontal(|ui| {
        super::modal_size::grip(f.kind, ui);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let label = if f.busy { "Creating…" } else { "Create" };
            if ui
                .add_enabled_ui(!f.busy, |ui| kit::primary_button(ui, label))
                .inner
                .clicked()
            {
                send(out, ModalMsg::Submit);
            }
            if ui.button("Cancel").clicked() {
                send(out, ModalMsg::Close);
            }
            let mut another = f.another;
            if ui.checkbox(&mut another, "Create another").changed() {
                send(out, ModalMsg::SetAnother(another));
            }
        });
    });
}

/// 'Discard what you wrote?' with the two ways out, shown when closing was asked for with something written.
fn discard_question(ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let warning = crate::theme::palette_of(ui).warning;
    ui.add_space(6.0);
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.colored_label(warning, "Discard what you wrote? It will be lost.");
        ui.horizontal(|ui| {
            if ui.button("Keep editing").clicked() {
                send(out, ModalMsg::KeepEditing);
            }
            if ui.button("Discard").clicked() {
                send(out, ModalMsg::Discard);
            }
        });
    });
}

/// The dialog's content.
pub fn create(state: &State, f: &CreateForm, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    header(state, f, ui, out);
    if f.confirm_discard {
        discard_question(ui, out);
    }
    ui.add_space(6.0);
    let room = (ui.ctx().content_rect().height() - 140.0).max(240.0);
    ui.style_mut().animation_time = 0.0;
    egui::ScrollArea::vertical()
        .max_height(room)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            title_field(f, ui, out);
            editors(f, ui, out);
            fields(state, f, ui, out);
            if let Some(e) = &f.error {
                ui.colored_label(ui.visuals().error_fg_color, e);
            }
        });
    footer(f, ui, out);
}
