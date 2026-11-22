//! The homework planner screen (STORY-0167): status tabs, the list with urgency and countdowns,
//! and the create and edit dialog.

use eframe::egui::{self, RichText};
use studyfrog_calendar::homework::{
    HomeworkItem, HomeworkPriority, HomeworkStatus, checklist_progress,
};
use studyfrog_calendar::homework_query::{
    HOMEWORK_TABS, countdown, planner_order, subjects, tab_filter, urgency,
};
use studyfrog_calendar::timetable::lessons_note;
use studyfrog_core::civil::LocalTime;
use studyfrog_core::uid::Uid;

use crate::homework_msg::HomeworkMsg;
use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::calendar_timetable::timetable_panel;
use crate::view::homework_marks::{urgency_colour, urgency_name};
use crate::view::{homework_form_view, kit};

/// Below this width the timetable goes above the list instead of beside it.
const SIDE_BY_SIDE_MIN_WIDTH: f32 = 640.0;

/// "All", then one chip per subject the homework belongs to. The chosen subject stays chosen when
/// the status tab changes.
fn subject_chips(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let current = state.planner.homework_ui.subject;
    let mut stacks: Vec<(Uid, String)> = subjects(&state.planner.homework)
        .into_iter()
        .map(|uid| {
            let name =
                course_name(state, Some(uid)).unwrap_or_else(|| "Unnamed subject".to_owned());
            (uid, name)
        })
        .collect();
    stacks.sort_by(|a, b| a.1.cmp(&b.1));
    ui.horizontal_wrapped(|ui| {
        ui.label("Subject:");
        let all = current.is_none();
        if ui
            .selectable_label(all, "All")
            .on_hover_text("Shows the homework of every subject.")
            .clicked()
            && !all
        {
            out.push(Msg::Homework(HomeworkMsg::Subject(None)));
        }
        for (uid, name) in &stacks {
            let active = current == Some(*uid);
            if ui
                .selectable_label(active, name)
                .on_hover_text("Shows only the homework of this subject.")
                .clicked()
                && !active
            {
                out.push(Msg::Homework(HomeworkMsg::Subject(Some(*uid))));
            }
        }
        if stacks.is_empty() {
            ui.label("No subjects yet: give an item a stack.");
        }
    });
}

fn send(out: &mut Vec<Msg>, m: HomeworkMsg) {
    out.push(Msg::Homework(m));
}

fn tab_name(tab: studyfrog_calendar::homework_query::HomeworkTab) -> &'static str {
    use studyfrog_calendar::homework_query::HomeworkTab;
    match tab {
        HomeworkTab::All => "All",
        HomeworkTab::Active => "Active",
        HomeworkTab::DueToday => "Due today",
        HomeworkTab::Overdue => "Overdue",
        HomeworkTab::Completed => "Completed",
    }
}

fn priority_name(p: HomeworkPriority) -> &'static str {
    match p {
        HomeworkPriority::Low => "Low priority",
        HomeworkPriority::Medium => "Medium priority",
        HomeworkPriority::High => "High priority",
    }
}

/// "45 min", or "2 h 30 min" for longer estimates.
pub fn estimate_text(minutes: u32) -> String {
    if minutes < 60 {
        format!("{minutes} min")
    } else if minutes.is_multiple_of(60) {
        format!("{} h", minutes / 60)
    } else {
        format!("{} h {} min", minutes / 60, minutes % 60)
    }
}

/// The title of the stack (subject) with this id, if it is still in the vault.
pub(crate) fn course_name(state: &State, course: Option<Uid>) -> Option<String> {
    let uid = course?;
    let v = state.vault.as_ref()?;
    v.overview
        .stacks
        .iter()
        .find(|s| s.uid == uid)
        .map(|s| s.title.clone())
}

/// The label of the button that moves an item to its next status.
fn advance_label(status: HomeworkStatus) -> &'static str {
    match status {
        HomeworkStatus::NotStarted => "Start",
        HomeworkStatus::InProgress => "Complete",
        HomeworkStatus::Completed => "Reopen",
    }
}

fn row(state: &State, item: &HomeworkItem, now: LocalTime, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let pal = palette_of(ui);
    let u = urgency(item, now);
    kit::card(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(&item.title).strong());
            kit::pill(ui, urgency_name(u), urgency_colour(&pal, u));
            ui.label(countdown(item, now));
        });
        ui.label(lessons_note(&state.planner.timetable, item.due.date));
        ui.horizontal_wrapped(|ui| {
            if let Some(name) = course_name(state, item.course) {
                kit::pill(ui, &name, pal.accent);
            }
            ui.label(priority_name(item.priority));
            ui.label(format!("About {}", estimate_text(item.estimate_min)));
            let (done, total) = checklist_progress(item);
            if total > 0 {
                ui.label(format!("{done} of {total} steps done"));
            }
        });
        for (index, step) in item.checklist.iter().enumerate() {
            let mut ticked = step.done;
            if ui.checkbox(&mut ticked, &step.text).changed() {
                send(out, HomeworkMsg::ToggleStep(item.id, index));
            }
        }
        ui.horizontal(|ui| {
            if kit::quiet_button(ui, advance_label(item.status))
                .on_hover_text("Moves this item to its next status.")
                .clicked()
            {
                send(out, HomeworkMsg::Advance(item.id));
            }
            if kit::quiet_button(ui, "Edit")
                .on_hover_text("Opens this for editing.")
                .clicked()
            {
                send(out, HomeworkMsg::EditItem(item.id));
            }
        });
    });
    ui.add_space(6.0);
}

/// Draw the homework screen.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.heading("Homework");
    let Some(now) = state.planner.homework_now else {
        ui.label("Loading homework…");
        return;
    };
    let pal = palette_of(ui);
    if let Some(e) = &state.planner.homework_ui.error {
        ui.colored_label(pal.danger, e);
    }
    ui.horizontal_wrapped(|ui| {
        for tab in HOMEWORK_TABS {
            let active = state.planner.homework_ui.tab == tab;
            if ui.selectable_label(active, tab_name(tab)).clicked() && !active {
                send(out, HomeworkMsg::Tab(tab));
            }
        }
        if kit::primary_button(ui, "+ New homework")
            .on_hover_text("Starts a new homework item.")
            .clicked()
        {
            send(out, HomeworkMsg::New);
        }
    });
    subject_chips(state, ui, out);
    ui.add_space(8.0);
    if let Some(form) = &state.planner.homework_ui.form {
        // Scrolls when the window is short, so the delete row and the buttons below the fields stay reachable.
        let height = ui.available_height().min(480.0);
        egui::ScrollArea::vertical()
            .id_salt("homework-dialog")
            .max_height(height)
            .auto_shrink([false, true])
            .show(ui, |ui| homework_form_view::draw(state, form, ui, out));
        ui.add_space(8.0);
    }
    let ui_state = &state.planner.homework_ui;
    let filter = tab_filter(ui_state.tab, ui_state.subject);
    let items = planner_order(&state.planner.homework, &filter, now);
    if ui.available_width() >= SIDE_BY_SIDE_MIN_WIDTH {
        ui.columns(2, |cols| {
            list(state, &items, now, &mut cols[0], out);
            timetable_panel(state, &mut cols[1]);
        });
    } else {
        egui::ScrollArea::vertical()
            .id_salt("homework-timetable")
            .max_height(200.0)
            .auto_shrink([false, true])
            .show(ui, |ui| timetable_panel(state, ui));
        ui.add_space(8.0);
        list(state, &items, now, ui, out);
    }
}

/// The filtered items, with the urgency and the lessons on each due date.
fn list(
    state: &State,
    items: &[HomeworkItem],
    now: LocalTime,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    if items.is_empty() {
        ui.label("Nothing here yet.");
        return;
    }
    egui::ScrollArea::vertical()
        .id_salt("homework-list")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for item in items {
                row(state, item, now, ui, out);
            }
        });
}
