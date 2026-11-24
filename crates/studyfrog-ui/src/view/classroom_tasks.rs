//! The assignments panel (TASK-0294): solo and group tasks with status badges and rosters, the
//! buttons to open a task and to set group work, and the drawer of the open task.

use eframe::egui::{self, Button, RichText};
use studyfrog_core::classroom::{Assignment, AssignmentKind, ClassroomSession};
use studyfrog_core::classroom_ops::participant_name;
use studyfrog_core::classroom_status::{WorkStatus, task_status};

use crate::classroom_types::ClassroomMsg;
use crate::classroom_ui::user_index;
use crate::msg::Msg;
use crate::state::State;
use crate::theme::{palette_of, tint};
use crate::view::classroom_drawer;

fn send(out: &mut Vec<Msg>, m: ClassroomMsg) {
    out.push(Msg::Classroom(m));
}

/// The assignments panel: active tasks first, then the completed ones, then the drawer of the open task.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(session) = state.classroom.session.as_ref() else {
        return;
    };
    ui.label(RichText::new("Assignments").strong());
    let assign = ui
        .add_enabled(!state.classroom.busy, Button::new("Assign group task"))
        .on_hover_text(
            "The teacher splits the students into groups of two and sets one task for each.",
        );
    if assign.clicked() {
        send(out, ClassroomMsg::AssignGroups);
    }
    if session.assignments.is_empty() {
        ui.weak("No tasks yet. The teacher sets one when you ask for it.");
    }
    let user = user_index(session);
    let (active, done): (Vec<&Assignment>, Vec<&Assignment>) = session
        .assignments
        .iter()
        .partition(|a| task_status(session, a.id, user) != WorkStatus::Graded);
    section(session, user, "Active", &active, ui, out);
    section(session, user, "Completed", &done, ui, out);
    if state.classroom.open_task.is_some() {
        ui.separator();
        classroom_drawer::draw(state, ui, out);
    }
}

fn section(
    session: &ClassroomSession,
    user: Option<usize>,
    title: &str,
    tasks: &[&Assignment],
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    if tasks.is_empty() {
        return;
    }
    ui.add_space(6.0);
    ui.label(RichText::new(title).small().strong());
    for task in tasks {
        row(session, user, task, ui, out);
    }
}

/// One task: its title, solo or group badge, status badge, roster and the button that opens it.
fn row(
    session: &ClassroomSession,
    user: Option<usize>,
    task: &Assignment,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    let pal = palette_of(ui);
    egui::Frame::new()
        .fill(pal.surface)
        .corner_radius(6)
        .inner_margin(8)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(&task.title).strong());
                kind_badge(ui, task);
                status_badge(ui, task_status(session, task.id, user));
            });
            ui.weak(roster(session, task));
            let open = ui
                .add(Button::new(format!("Open {}", task.title)).small())
                .on_hover_text("Opens this task to see its prompt, roster and grade.");
            if open.clicked() {
                send(out, ClassroomMsg::OpenTask(Some(task.id)));
            }
        });
    ui.add_space(4.0);
}

/// "Solo" or "Group", whose accessible name says which kind of task it marks.
fn kind_badge(ui: &mut egui::Ui, task: &Assignment) {
    let (text, access) = match task.kind {
        AssignmentKind::Solo { .. } => ("Solo", "Solo task"),
        AssignmentKind::Group { .. } => ("Group", "Group task"),
    };
    tag(
        ui,
        text,
        access,
        tint(palette_of(ui).alt, 40),
        palette_of(ui).alt,
    );
}

/// The status badge: `assigned`, `submitted` or `graded`, named "Status: ..." for assistive tech.
pub fn status_badge(ui: &mut egui::Ui, status: WorkStatus) {
    let pal = palette_of(ui);
    let (text, color) = match status {
        WorkStatus::Assigned => ("assigned", pal.text_weak),
        WorkStatus::Submitted => ("submitted", pal.alt),
        WorkStatus::Graded => ("graded", pal.success),
    };
    tag(ui, text, &format!("Status: {text}"), tint(color, 40), color);
}

/// A small tinted tag whose visible text is also its accessible name.
fn tag(ui: &mut egui::Ui, text: &str, access: &str, fill: egui::Color32, color: egui::Color32) {
    let tag = egui::Frame::new()
        .fill(fill)
        .corner_radius(4)
        .inner_margin(egui::Margin::symmetric(6, 2))
        .show(ui, |ui| {
            ui.label(RichText::new(text).small().strong().color(color))
        })
        .inner;
    tag.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, true, access));
}

/// Who the task is for: the assignee, or each group with its members.
fn roster(session: &ClassroomSession, task: &Assignment) -> String {
    match &task.kind {
        AssignmentKind::Solo { assignee } => {
            format!("Assignee: {}", names(session, &[*assignee]))
        }
        AssignmentKind::Group { groups } => groups
            .iter()
            .map(|g| format!("{}: {}", g.name, names(session, &g.members)))
            .collect::<Vec<_>>()
            .join("; "),
    }
}

fn names(session: &ClassroomSession, members: &[usize]) -> String {
    members
        .iter()
        .map(|&m| session.participants.get(m).map_or("?", participant_name))
        .collect::<Vec<_>>()
        .join(", ")
}
