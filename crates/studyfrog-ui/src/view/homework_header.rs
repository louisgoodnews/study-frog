//! The homework headline and tab row in the redesign style (EPIC-073, STORY-0236): the title and a summary of the
//! tab in view on the gradient panel, then the status tabs with the rail highlight. The same tabs, messages and
//! labels as before; only the drawing changes.

use eframe::egui::{self, RichText};
use studyfrog_calendar::homework_query::{HOMEWORK_TABS, HomeworkTab, planner_order, tab_filter};
use studyfrog_core::civil::LocalTime;
use studyfrog_core::count::counted;

use crate::homework_msg::HomeworkMsg;
use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;
use crate::theme::style::style_of;
use crate::view::{gradient_panel, kit};

/// The height of the headline panel, in points.
const HEADER_HEIGHT: f32 = 96.0;

/// The name of a status tab.
fn tab_name(tab: HomeworkTab) -> &'static str {
    match tab {
        HomeworkTab::All => "All",
        HomeworkTab::Active => "Active",
        HomeworkTab::DueToday => "Due today",
        HomeworkTab::Overdue => "Overdue",
        HomeworkTab::Completed => "Completed",
    }
}

/// "Active: 4 items" for the tab and subject in view, or a loading note before the clock is known.
fn summary(state: &State, now: Option<LocalTime>) -> String {
    let Some(now) = now else {
        return "Loading homework…".to_owned();
    };
    let ui_state = &state.planner.homework_ui;
    let filter = tab_filter(ui_state.tab, ui_state.subject);
    let count = planner_order(&state.planner.homework, &filter, now).len();
    format!("{}: {}", tab_name(ui_state.tab), counted(count, "item", "items"))
}

/// The title and the summary of the tab in view, on the gradient panel.
pub(super) fn headline(state: &State, now: Option<LocalTime>, ui: &mut egui::Ui) {
    let text = summary(state, now);
    gradient_panel::panel(ui, HEADER_HEIGHT, |ui, s| {
        ui.label(
            RichText::new("Homework")
                .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                .strong()
                .color(s.gradient_text),
        );
        ui.label(RichText::new(text).color(s.gradient_text));
    });
}

/// The status tabs, with the rail highlight on the one in view, then the action that starts a new item.
pub(super) fn tab_row(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let current = state.planner.homework_ui.tab;
    let p = palette_of(ui);
    let s = style_of(&p);
    ui.horizontal_wrapped(|ui| {
        for tab in HOMEWORK_TABS {
            let selected = tab == current;
            let text = if selected { p.accent } else { p.text };
            let fill = if selected { s.rail_active } else { s.rail };
            let button = egui::Button::new(RichText::new(tab_name(tab)).color(text)).fill(fill);
            if ui
                .add(button)
                .on_hover_text("Shows the homework with this status.")
                .clicked()
                && !selected
            {
                out.push(Msg::Homework(HomeworkMsg::Tab(tab)));
            }
        }
        if kit::primary_button(ui, "+ New homework")
            .on_hover_text("Starts a new homework item.")
            .clicked()
        {
            out.push(Msg::Homework(HomeworkMsg::New));
        }
    });
}
