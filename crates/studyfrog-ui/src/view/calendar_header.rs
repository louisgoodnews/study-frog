//! The calendar's headline and view switcher (EPIC-073, STORY-0228): the title and the period on the gradient panel,
//! then the view modes and, outside the month view, the period buttons. The modes and buttons are the same as before.

use eframe::egui::{self, RichText};
use studyfrog_core::civil::format_date;

use crate::calendar_mode::{CALENDAR_VIEW_MODES, CalendarViewMode, calendar_view_mode_name};
use crate::msg::Msg;
use crate::planner::PlanMsg;
use crate::state::State;
use crate::view::{calendar_grid, gradient_panel};

const HEADLINE_HEIGHT: f32 = 64.0;

fn plan(out: &mut Vec<Msg>, m: PlanMsg) {
    out.push(Msg::Plan(m));
}

/// The period the view shows: the month, the chosen day, or the timetable's name.
pub(super) fn period(state: &State) -> String {
    let p = &state.planner;
    match p.view_mode {
        CalendarViewMode::Month => calendar_grid::month_label(state),
        CalendarViewMode::Week | CalendarViewMode::Day => format_date(p.selected),
        CalendarViewMode::Timetable => "Weekly timetable".to_owned(),
    }
}

/// The screen title and the period, on the gradient panel. Only the whole calendar has it; a stack's calendar sits
/// under the stack's own headline.
pub(super) fn headline(state: &State, ui: &mut egui::Ui) {
    let text = period(state);
    gradient_panel::panel(ui, HEADLINE_HEIGHT, |ui, s| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Calendar")
                    .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                    .strong()
                    .color(s.gradient_text),
            );
            ui.label(RichText::new(text).color(s.gradient_text));
        });
    });
}

/// The view modes, then the period buttons when the view is not the month (the month has its own in its grid).
pub(super) fn switcher(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let p = &state.planner;
    ui.horizontal(|ui| {
        for mode in CALENDAR_VIEW_MODES {
            let active = p.view_mode == mode;
            let name = calendar_view_mode_name(mode);
            let tip = format!("Shows the calendar as {name}.");
            if ui.selectable_label(active, name).on_hover_text(tip).clicked() && !active {
                plan(out, PlanMsg::SetViewMode(mode));
            }
        }
        if p.view_mode != CalendarViewMode::Month {
            ui.add_space(12.0);
            if ui
                .button("<")
                .on_hover_text("Goes to the period before this one.")
                .clicked()
            {
                plan(out, PlanMsg::ShiftPeriod(-1));
            }
            if ui.button("Today").on_hover_text("Goes to today.").clicked() {
                plan(out, PlanMsg::Today);
            }
            if ui
                .button(">")
                .on_hover_text("Goes to the period after this one.")
                .clicked()
            {
                plan(out, PlanMsg::ShiftPeriod(1));
            }
        }
    });
}
