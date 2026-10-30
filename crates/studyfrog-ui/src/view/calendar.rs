//! The calendar: a month grid, the sessions of the chosen day, and the session form.

use eframe::egui::{self, Grid, RichText};
use studyfrog_calendar::session::{Occurrence, Repeat, Session};
use studyfrog_core::civil::{Date, format_date, format_minute};
use studyfrog_core::uid::Uid;

use crate::msg::Msg;
use crate::planner::{PlanMsg, Planner, grid_of};
use crate::state::State;
use crate::view::session_form;

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const DAY_NAMES: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

fn plan(out: &mut Vec<Msg>, m: PlanMsg) {
    out.push(Msg::Plan(m));
}

/// Month header, grid, and the side panel with the day's sessions or the form.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let p = &state.planner;
    ui.heading("Calendar");
    if !p.loaded {
        ui.label("Loading sessions…");
        return;
    }
    ui.horizontal(|ui| {
        if ui.button("Previous month").clicked() {
            plan(out, PlanMsg::Month(-1));
        }
        ui.strong(format!(
            "{} {}",
            MONTHS[usize::from(p.month.1) - 1],
            p.month.0
        ));
        if ui.button("Next month").clicked() {
            plan(out, PlanMsg::Month(1));
        }
        if ui.button("Today").clicked() {
            plan(out, PlanMsg::Today);
        }
    });
    ui.columns(2, |cols| {
        month_grid(p, &mut cols[0], out);
        match &p.form {
            Some(form) => session_form::draw(state, form, &mut cols[1], out),
            None => day_panel(state, &mut cols[1], out),
        }
    });
}

fn count_on(p: &Planner, day: Date) -> usize {
    p.shown.iter().filter(|o| o.date == day).count()
}

fn month_grid(p: &Planner, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    Grid::new("month").num_columns(7).show(ui, |ui| {
        for name in DAY_NAMES {
            ui.strong(name);
        }
        ui.end_row();
        for (i, day) in grid_of(p.month).into_iter().enumerate() {
            let n = count_on(p, day);
            let mut text = RichText::new(if n > 0 {
                format!("{} · {n}", day.day)
            } else {
                day.day.to_string()
            });
            if (day.year, day.month) != p.month {
                text = text.weak();
            }
            if day == p.today {
                text = text.strong().underline();
            }
            let response = ui.selectable_label(day == p.selected, text);
            if response.on_hover_text(format_date(day)).clicked() {
                plan(out, PlanMsg::Select(day));
            }
            if i % 7 == 6 {
                ui.end_row();
            }
        }
    });
}

fn stack_title(state: &State, uid: Option<Uid>) -> Option<String> {
    let uid = uid?;
    let v = state.vault.as_ref()?;
    v.overview
        .stacks
        .iter()
        .find(|s| s.uid == uid)
        .map(|s| s.title.clone())
}

fn day_panel(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let p = &state.planner;
    ui.heading(format!("Sessions on {}", format_date(p.selected)));
    let today: Vec<&Occurrence> = p.shown.iter().filter(|o| o.date == p.selected).collect();
    if today.is_empty() {
        ui.label("Nothing planned for this day.");
    }
    for o in today {
        if let Some(s) = p.sessions.iter().find(|s| s.id == o.session) {
            occurrence_row(state, s, o, ui, out);
        }
    }
    if ui.button("Add session").clicked() {
        plan(out, PlanMsg::New);
    }
}

fn occurrence_row(
    state: &State,
    s: &Session,
    o: &Occurrence,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    let end = o.start_minute + o.duration_min;
    let when = format!(
        "{}–{}",
        format_minute(o.start_minute),
        format_minute(end % 1440)
    );
    let stack = stack_title(state, s.stack).map_or(String::new(), |t| format!(" · {t}"));
    ui.group(|ui| {
        ui.label(RichText::new(&s.title).strong());
        ui.label(format!("{when}{stack}"));
        ui.horizontal_wrapped(|ui| {
            if ui.button(format!("Edit {}", s.title)).clicked() {
                plan(out, PlanMsg::Edit(s.id));
            }
            if ui.button(format!("Delete {}", s.title)).clicked() {
                plan(out, PlanMsg::Delete(s.id));
            }
            if s.repeat != Repeat::Never && ui.button(format!("Skip {} today", s.title)).clicked() {
                plan(out, PlanMsg::Skip(s.id, o.date));
            }
            if ui
                .button(format!("Start rehearsal for {}", s.title))
                .clicked()
            {
                plan(out, PlanMsg::Rehearse(s.id));
            }
        });
    });
}
