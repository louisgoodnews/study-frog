//! The calendar: a month grid that fills the window, and beside it the chosen day, what is coming up
//! and the entry form. The headline and the view switcher are in calendar_header, the side panel in calendar_side.

use eframe::egui::{self, vec2};
use studyfrog_core::uid::Uid;

use crate::calendar_mode::{CalendarViewMode, calendar_view_mode_name};
use crate::msg::Msg;
use crate::state::State;
use crate::theme::PAD_CARD;
use crate::view::{calendar_grid, calendar_header, calendar_side, home_hero, session_form};

const SIDE_WIDTH: f32 = 380.0;

/// The month on the left, the day and the form on the right.
/// The whole screen.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    draw_scoped(state, None, ui, out);
}

/// The calendar of one stack (`Some`) or of everything (`None`): the same month grid, day list and form.
pub fn draw_scoped(state: &State, scope: Option<Uid>, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let _timer = studyfrog_debug::timed("ui", "calendar draw");
    let p = &state.planner;
    if !p.loaded {
        ui.heading("Calendar");
        ui.label("Loading sessions…");
        return;
    }
    let name = calendar_view_mode_name(p.view_mode);
    studyfrog_log::log_debug!("ui", "calendar view shown"; view = name);
    if scope.is_none() {
        calendar_header::headline(state, ui);
        ui.add_space(8.0);
    }
    calendar_header::switcher(state, ui, out);
    ui.add_space(8.0);

    match p.view_mode {
        CalendarViewMode::Month => draw_month_layout(state, scope, ui, out),
        CalendarViewMode::Week => super::calendar_week::draw(state, scope, ui, out),
        CalendarViewMode::Day => super::calendar_day::draw(state, scope, ui, out),
        CalendarViewMode::Timetable => super::calendar_timetable::draw(state, ui, out),
    }
}

fn draw_month_layout(state: &State, scope: Option<Uid>, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let p = &state.planner;
    let (total, height) = (ui.available_width(), ui.available_height());
    let side_width = SIDE_WIDTH.min(total * 0.45);
    let month_width = total - side_width - ui.spacing().item_spacing.x;
    ui.horizontal_top(|ui| {
        let down = egui::Layout::top_down(egui::Align::Min);
        ui.allocate_ui_with_layout(vec2(month_width, height), down, |ui| {
            ui.set_min_size(vec2(month_width, height));
            home_hero::card(ui, |ui| {
                ui.set_width(month_width - 2.0 * PAD_CARD);
                calendar_grid::month(state, scope, ui, out);
            });
        });
        ui.allocate_ui_with_layout(vec2(side_width, height), down, |ui| {
            ui.set_min_size(vec2(side_width, height));
            egui::ScrollArea::vertical().show(ui, |ui| match &p.form {
                Some(form) => session_form::draw(state, form, ui, out),
                None => calendar_side::side(state, scope, ui, out),
            });
        });
    });
}
