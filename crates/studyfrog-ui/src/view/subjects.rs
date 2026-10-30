//! The teachers and subjects section of the settings screen.

use eframe::egui::{self, ComboBox, RichText, Role, WidgetInfo};

use crate::msg::Msg;
use crate::state::State;
use crate::subjects_ui::SubjectsMsg;
use crate::theme::palette_of;
use crate::view::kit;
use crate::view::widgets::text_field;

fn send(out: &mut Vec<Msg>, m: SubjectsMsg) {
    out.push(Msg::Subjects(m));
}

/// A drop-down of the teachers (and "No teacher"), named for assistive technology.
fn teacher_picker(
    ui: &mut egui::Ui,
    (name, teachers): (&str, &[String]),
    current: Option<&str>,
) -> Option<Option<String>> {
    let mut picked = None;
    let r = ComboBox::from_id_salt(("teacher-of", name))
        .selected_text(current.unwrap_or("No teacher"))
        .show_ui(ui, |ui| {
            if ui
                .selectable_label(current.is_none(), "No teacher")
                .clicked()
            {
                picked = Some(None);
            }
            for t in teachers {
                if ui
                    .selectable_label(current == Some(t.as_str()), t)
                    .clicked()
                {
                    picked = Some(Some(t.clone()));
                }
            }
        });
    r.response
        .widget_info(|| WidgetInfo::labeled(Role::ComboBox, true, format!("Teacher of {name}")));
    picked
}

/// The section: teachers, then subjects with their teacher.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let s = &state.subjects;
    let weak = palette_of(ui).text_weak;
    kit::section(ui, "Subjects and teachers");
    if let Some(e) = &s.error {
        ui.colored_label(ui.visuals().error_fg_color, e);
    }
    ui.label(RichText::new("A subject has one teacher. A stack can have several subjects, and what is inside a stack takes its subjects.").color(weak));
    ui.add_space(4.0);
    ui.label(RichText::new("Teachers").strong());
    for t in &s.book.teachers {
        ui.horizontal(|ui| {
            ui.label(t);
            if ui.small_button(format!("Remove teacher {t}")).clicked() {
                send(out, SubjectsMsg::RemoveTeacher(t.clone()));
            }
        });
    }
    if let Some(t) = text_field(ui, "New teacher", &s.teacher_draft, "name", 1) {
        send(out, SubjectsMsg::TeacherDraft(t));
    }
    if ui.button("Add teacher").clicked() {
        send(out, SubjectsMsg::AddTeacher);
    }
    ui.add_space(8.0);
    ui.label(RichText::new("Subjects").strong());
    for sub in &s.book.subjects {
        ui.horizontal(|ui| {
            ui.label(&sub.name);
            if let Some(new) =
                teacher_picker(ui, (&sub.name, &s.book.teachers), sub.teacher.as_deref())
            {
                send(out, SubjectsMsg::SetTeacher(sub.name.clone(), new));
            }
            if ui
                .small_button(format!("Remove subject {}", sub.name))
                .clicked()
            {
                send(out, SubjectsMsg::RemoveSubject(sub.name.clone()));
            }
        });
    }
    if let Some(t) = text_field(ui, "New subject", &s.subject_draft, "name", 1) {
        send(out, SubjectsMsg::SubjectDraft(t));
    }
    ui.horizontal(|ui| {
        ui.label("Taught by");
        if let Some(new) = teacher_picker(
            ui,
            ("the new subject", &s.book.teachers),
            s.subject_teacher.as_deref(),
        ) {
            send(out, SubjectsMsg::SubjectTeacher(new));
        }
        if ui.button("Add subject").clicked() {
            send(out, SubjectsMsg::AddSubject);
        }
    });
}
