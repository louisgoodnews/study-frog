//! The task drawer (TASK-0294): the open task's prompt and rubric, the group thread for group work, the
//! answer editor with Turn in, and the teacher's grade. Every action is a message; nothing is changed here.

use eframe::egui::{self, Button, RichText, TextEdit};
use studyfrog_core::classroom::{Assignment, AssignmentKind, ChatTurn, ClassroomSession, Owner};
use studyfrog_core::classroom_ops::participant_name;
use studyfrog_core::classroom_status::{grade_for, owner_of, submission_for, task_status};

use crate::classroom_types::ClassroomMsg;
use crate::classroom_ui::user_index;
use crate::classroom_work_ui::draft_of;
use crate::msg::Msg;
use crate::state::State;
use crate::view::classroom_grade;
use crate::view::classroom_tasks::status_badge;

fn send(out: &mut Vec<Msg>, m: ClassroomMsg) {
    out.push(Msg::Classroom(m));
}

/// The drawer of the open task, if the session has it.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let (Some(session), Some(id)) = (state.classroom.session.as_ref(), state.classroom.open_task)
    else {
        return;
    };
    let Some(task) = session.assignments.iter().find(|a| a.id == id) else {
        return;
    };
    let user = user_index(session);
    let owner = user.and_then(|u| owner_of(session, id, u));
    header(session, task, user, ui, out);
    match owner {
        Some(owner) => {
            if let Owner::Group(group) = owner {
                thread(state, session, task, group, ui, out);
            }
            answer(state, session, task, owner, ui, out);
        }
        None => {
            ui.weak("This task is set for other students. You can read it, but not answer it.");
        }
    }
    if let Some(grade) = shown_owner(task, owner).and_then(|o| grade_for(session, id, o)) {
        ui.add_space(6.0);
        classroom_grade::draw(ui, grade);
    }
}

/// The owner whose grade is shown: the user's own work, else a solo task's assignee. A group task
/// with no user in it has no single answer to show.
fn shown_owner(task: &Assignment, own: Option<Owner>) -> Option<Owner> {
    own.or(match task.kind {
        AssignmentKind::Solo { assignee } => Some(Owner::Participant(assignee)),
        AssignmentKind::Group { .. } => None,
    })
}

/// The title, the close button, the status, the task text and its rubric.
fn header(
    session: &ClassroomSession,
    task: &Assignment,
    user: Option<usize>,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(&task.title).strong());
        if ui
            .add(Button::new("Close task").small())
            .on_hover_text("Closes this task and returns to the list.")
            .clicked()
        {
            send(out, ClassroomMsg::OpenTask(None));
        }
    });
    status_badge(ui, task_status(session, task.id, user));
    ui.add(egui::Label::new(&task.prompt).wrap());
    ui.weak(format!("Rubric: {}", task.rubric.join("; ")));
}

/// The group thread: the members' messages, and the composer that writes to it as the user.
fn thread(
    state: &State,
    session: &ClassroomSession,
    task: &Assignment,
    group: usize,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    let AssignmentKind::Group { groups } = &task.kind else {
        return;
    };
    let Some(roster) = groups.get(group) else {
        return;
    };
    ui.add_space(6.0);
    ui.label(
        RichText::new(format!("{} thread", roster.name))
            .small()
            .strong(),
    );
    egui::ScrollArea::vertical()
        .id_salt("classroom-group-thread")
        .max_height(160.0)
        .auto_shrink([false, true])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            let lines: Vec<&ChatTurn> = session
                .turns
                .iter()
                .filter(|t| roster.members.contains(&t.speaker))
                .collect();
            if lines.is_empty() {
                ui.weak("No messages yet. Write to your group; your teammates will answer.");
            }
            for turn in lines {
                let who = session
                    .participants
                    .get(turn.speaker)
                    .map_or("?", participant_name);
                ui.add(egui::Label::new(format!("{who}: {}", turn.text)).wrap());
            }
        });
    composer(state, ui, out);
}

/// The group composer: text for the group thread, sent as the user.
fn composer(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let busy = state.classroom.busy;
    let mut text = state.classroom.group_input.clone();
    ui.horizontal(|ui| {
        let width = (ui.available_width() - 120.0).max(120.0);
        let resp = ui.add(
            TextEdit::singleline(&mut text)
                .desired_width(width)
                .hint_text("Message your group"),
        );
        resp.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::TextInput, true, "Message to your group")
        });
        if resp.changed() {
            send(out, ClassroomMsg::GroupInput(text.clone()));
        }
        let can_send = !busy && !text.trim().is_empty();
        let send_btn = ui
            .add_enabled(can_send, Button::new("Send to group"))
            .on_hover_text("Your teammates answer after your message.");
        if can_send && send_btn.clicked() {
            send(out, ClassroomMsg::SendGroup);
        }
    });
}

/// The answer for the user's own work: the editor and Turn in while it is a draft, the locked text
/// and the request for a grade once it is turned in.
fn answer(
    state: &State,
    session: &ClassroomSession,
    task: &Assignment,
    owner: Owner,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    let busy = state.classroom.busy;
    ui.add_space(6.0);
    let Some(sub) = submission_for(session, task.id, owner) else {
        draft_editor(state, task, ui, out);
        return;
    };
    ui.label(
        RichText::new("Your answer (turned in, locked)")
            .small()
            .strong(),
    );
    ui.add(egui::Label::new(&sub.text).wrap());
    if sub.grade.is_none() {
        let ask = ui
            .add_enabled(!busy, Button::new("Ask the teacher to grade"))
            .on_hover_text("The teacher grades the answer against the rubric.");
        if ask.clicked() {
            send(out, ClassroomMsg::AskGrade);
        }
    }
}

/// The shared answer as drafted: the editor and the Turn in button, which needs a non-blank draft.
fn draft_editor(state: &State, task: &Assignment, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let busy = state.classroom.busy;
    let mut text = draft_of(&state.classroom.drafts, task.id).to_owned();
    ui.label(RichText::new("Shared answer (draft)").small().strong());
    let resp = ui.add(
        TextEdit::multiline(&mut text)
            .desired_rows(5)
            .desired_width(ui.available_width())
            .hint_text("Draft the answer for your group here"),
    );
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::TextInput, true, "Your answer"));
    if resp.changed() {
        send(out, ClassroomMsg::DraftEdit(text.clone()));
    }
    let can_turn_in = !busy && !text.trim().is_empty();
    let turn_in = ui
        .add_enabled(can_turn_in, Button::new("Turn in"))
        .on_hover_text("Locks this answer. It cannot be changed after you turn it in.");
    if can_turn_in && turn_in.clicked() {
        send(out, ClassroomMsg::TurnIn);
    }
}
