//! The calendar: a month grid that fills the window, and beside it the chosen day, what is coming up
//! and the entry form.

use eframe::egui::{self, RichText, vec2};
use studyfrog_calendar::recurrence::occurrences_all;
use studyfrog_calendar::session::{EventKind, Occurrence, Repeat, Session, event_kind_name};
use studyfrog_core::civil::{add_days, format_date, format_minute};
use studyfrog_core::uid::Uid;

use crate::msg::Msg;
use crate::planner::PlanMsg;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::calendar_grid::{self, kind_color};
use crate::view::{kit, session_form};

const SIDE_WIDTH: f32 = 380.0;
const UPCOMING_DAYS: i64 = 14;

fn plan(out: &mut Vec<Msg>, m: PlanMsg) {
    out.push(Msg::Plan(m));
}

/// The month on the left, the day and the form on the right.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let p = &state.planner;
    if !p.loaded {
        ui.heading("Calendar");
        ui.label("Loading sessions…");
        return;
    }
    let (total, height) = (ui.available_width(), ui.available_height());
    let side_width = SIDE_WIDTH.min(total * 0.45);
    let month_width = total - side_width - ui.spacing().item_spacing.x;
    ui.horizontal_top(|ui| {
        let down = egui::Layout::top_down(egui::Align::Min);
        ui.allocate_ui_with_layout(vec2(month_width, height), down, |ui| {
            ui.set_min_size(vec2(month_width, height));
            calendar_grid::month(p, ui, out);
        });
        ui.allocate_ui_with_layout(vec2(side_width, height), down, |ui| {
            ui.set_min_size(vec2(side_width, height));
            egui::ScrollArea::vertical().show(ui, |ui| match &p.form {
                Some(form) => session_form::draw(state, form, ui, out),
                None => side(state, ui, out),
            });
        });
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

fn side(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let p = &state.planner;
    ui.label(
        RichText::new(format!("Sessions on {}", format_date(p.selected)))
            .size(18.0)
            .strong(),
    );
    let day: Vec<&Occurrence> = p.shown.iter().filter(|o| o.date == p.selected).collect();
    if day.is_empty() {
        ui.weak("Nothing planned for this day.");
    }
    for o in day {
        if let Some(s) = p.sessions.iter().find(|s| s.id == o.session) {
            occurrence_row(state, (s, o), ui, out);
        }
    }
    if kit::primary_button(ui, "Add to calendar").clicked() {
        plan(out, PlanMsg::New);
    }
    ui.add_space(10.0);
    kit::section(ui, "Coming up");
    upcoming(state, ui);
}

fn upcoming(state: &State, ui: &mut egui::Ui) {
    let p = &state.planner;
    let pal = palette_of(ui);
    let list = occurrences_all(&p.sessions, p.today, add_days(p.today, UPCOMING_DAYS));
    if list.is_empty() {
        ui.weak("Nothing in the next two weeks.");
    }
    for o in list.iter().take(12) {
        let Some(s) = p.sessions.iter().find(|s| s.id == o.session) else {
            continue;
        };
        ui.horizontal(|ui| {
            kit::pill(ui, event_kind_name(s.kind), kind_color(&pal, s.kind));
            ui.label(format!(
                "{} {}  {}",
                format_date(o.date),
                format_minute(o.start_minute),
                s.title
            ));
        });
    }
}

fn occurrence_row(
    state: &State,
    (s, o): (&Session, &Occurrence),
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
    let pal = palette_of(ui);
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            kit::pill(ui, event_kind_name(s.kind), kind_color(&pal, s.kind));
            ui.label(RichText::new(&s.title).strong());
        });
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
            if s.kind == EventKind::Study
                && ui
                    .button(format!("Start rehearsal for {}", s.title))
                    .clicked()
            {
                plan(out, PlanMsg::Rehearse(s.id));
            }
        });
    });
}
