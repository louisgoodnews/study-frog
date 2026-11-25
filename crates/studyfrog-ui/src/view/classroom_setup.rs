//! The form that starts a classroom session: title, sources, model, simulated students.

use eframe::egui::{self, Button, ComboBox, RichText, ScrollArea, Slider, TextEdit};
use studyfrog_agent::lecture_student::PRESET_PERSONAS;
use studyfrog_core::classroom::{SourceKind, SourceRef};
use studyfrog_service::classroom_material::pdf_source_uid;

use crate::classroom_types::ClassroomMsg;
use crate::classroom_ui::{MAX_STUDENTS, chosen_provider, is_chosen};
use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::home_hero;
use studyfrog_agent::provider::is_local;

fn send(out: &mut Vec<Msg>, m: ClassroomMsg) {
    out.push(Msg::Classroom(m));
}

/// The session form.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let c = &state.classroom;
    let pal = palette_of(ui);
    home_hero::card(ui, |ui| {
        ui.label(RichText::new("New classroom session").strong());
        ui.add_space(6.0);
        title_row(ui, &c.title, out);
        ui.add_space(8.0);
        ui.label(RichText::new("Teach from").small().strong());
        sources(state, ui, out);
        ui.add_space(8.0);
        model_row(state, ui, out);
        ui.add_space(8.0);
        students_row(state, ui, out);
        ui.add_space(12.0);
        let start = ui
            .add_enabled(!c.busy, Button::new("Start session"))
            .on_hover_text(
                "Creates the session. No model is called until you continue the lecture.",
            );
        if start.clicked() {
            send(out, ClassroomMsg::Start);
        }
        if let Some(error) = &c.error {
            ui.add_space(6.0);
            ui.label(RichText::new(error).color(pal.warning));
        }
    });
}

fn title_row(ui: &mut egui::Ui, title: &str, out: &mut Vec<Msg>) {
    ui.horizontal(|ui| {
        ui.label("Title");
        let mut text = title.to_owned();
        let resp = ui.add(
            TextEdit::singleline(&mut text)
                .hint_text("Classroom")
                .desired_width(260.0),
        );
        resp.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::TextInput, true, "Session title")
        });
        if resp.changed() {
            send(out, ClassroomMsg::Title(text));
        }
    });
}

/// The stacks and PDFs to tick. Notes are not offered yet.
fn sources(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let chosen = &state.classroom.sources;
    ScrollArea::vertical()
        .id_salt("classroom-sources")
        .max_height(180.0)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            let stacks = state.vault.iter().flat_map(|v| v.overview.stacks.iter());
            for stack in stacks {
                let source = SourceRef {
                    kind: SourceKind::Stack,
                    uid: stack.uid,
                };
                let mut on = is_chosen(chosen, source.kind, source.uid);
                if ui.checkbox(&mut on, &stack.title).changed() {
                    send(out, ClassroomMsg::ToggleSource(source));
                }
            }
            for row in &state.pdf_library.rows {
                let source = SourceRef {
                    kind: SourceKind::Pdf,
                    uid: pdf_source_uid(&row.path),
                };
                let mut on = is_chosen(chosen, source.kind, source.uid);
                if ui.checkbox(&mut on, &row.path).changed() {
                    send(out, ClassroomMsg::ToggleSource(source));
                }
            }
            let none_yet = state.pdf_library.rows.is_empty()
                && state.vault.iter().all(|v| v.overview.stacks.is_empty());
            if none_yet {
                ui.weak("Add a stack or a PDF to the vault first.");
            }
        });
}

fn model_row(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let providers = &state.agent.connections.providers;
    if providers.is_empty() {
        ui.weak("Connect a model in Assistant first.");
        return;
    }
    let current = chosen_provider(state).ok();
    let shown = current
        .as_ref()
        .map_or_else(String::new, |p| p.name.clone());
    ui.horizontal(|ui| {
        ui.label("Model");
        let combo = ComboBox::from_id_salt("classroom-model")
            .selected_text(shown)
            .show_ui(ui, |ui| {
                for p in providers {
                    let kind = if is_local(p) { "local" } else { "remote" };
                    let label = format!("{} ({kind}, {})", p.name, p.model);
                    let selected = current.as_ref().is_some_and(|c| c.name == p.name);
                    if ui
                        .selectable_label(selected, label)
                        .on_hover_text("Uses this model for the session.")
                        .clicked()
                    {
                        send(out, ClassroomMsg::ChooseProvider(p.name.clone()));
                    }
                }
            });
        combo
            .response
            .widget_info(|| egui::WidgetInfo::labeled(egui::Role::ComboBox, true, "Model"));
    });
}

fn students_row(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let mut n = state.classroom.students.min(MAX_STUDENTS);
    let slider = ui.add(Slider::new(&mut n, 0..=MAX_STUDENTS).text("Simulated students"));
    if slider.changed() {
        send(out, ClassroomMsg::Students(n));
    }
    let names: Vec<&str> = PRESET_PERSONAS.iter().take(n).map(|p| p.name).collect();
    let weak = palette_of(ui).text_weak;
    let line = if names.is_empty() {
        "No simulated students: the teacher and you only.".to_owned()
    } else {
        format!("Personas: {}", names.join(", "))
    };
    ui.label(RichText::new(line).small().color(weak));
}
