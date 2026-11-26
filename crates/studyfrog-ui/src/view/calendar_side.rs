//! The calendar's side panel (EPIC-073, STORY-0228): the chosen day's sessions, the planned reviews, what is coming up
//! and the entry button. The entries are cards in the redesign style; the actions are the same as before.

use eframe::egui::{self, RichText};
use studyfrog_calendar::recurrence::occurrences_all;
use studyfrog_calendar::session::{EventKind, Occurrence, Repeat, Session, event_kind_name};
use studyfrog_core::civil::{add_days, format_date, format_minute};
use studyfrog_core::uid::Uid;

use crate::msg::Msg;
use crate::planner::PlanMsg;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::calendar_grid::kind_color;
use crate::view::{home_hero, kit};

const UPCOMING_DAYS: i64 = 14;

fn plan(out: &mut Vec<Msg>, m: PlanMsg) {
    out.push(Msg::Plan(m));
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

/// The sessions of the chosen day, the add button, the planned reviews and the list of what is coming up.
pub(super) fn side(state: &State, scope: Option<Uid>, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let p = &state.planner;
    ui.label(
        RichText::new(format!("Sessions on {}", format_date(p.selected)))
            .size(crate::typo::size(ui, crate::typo::Tier::Heading))
            .strong(),
    );
    ui.add_space(4.0);
    let day: Vec<&Occurrence> = p
        .shown
        .iter()
        .filter(|o| o.date == p.selected)
        .filter(|o| {
            scope.is_none_or(|u| {
                p.sessions
                    .iter()
                    .any(|s| s.id == o.session && s.stack == Some(u))
            })
        })
        .collect();
    if day.is_empty() {
        ui.weak("Nothing planned for this day.");
    }
    for o in day {
        if let Some(s) = p.sessions.iter().find(|s| s.id == o.session) {
            occurrence_row(state, (s, o), ui, out);
        }
    }
    ui.add_space(8.0);
    if kit::primary_button(ui, "Add to calendar")
        .on_hover_text("Adds this to the calendar.")
        .clicked()
    {
        plan(out, PlanMsg::New);
    }
    ui.add_space(14.0);
    kit::section(ui, "Exam Scheduler");
    super::exam_scheduler::side_section(state, ui, out);
    ui.add_space(14.0);
    kit::section(ui, "Planned reviews");
    super::calendar_reviews::side_section(state, scope, ui, out);
    ui.add_space(14.0);
    kit::section(ui, "Coming up");
    upcoming(state, scope, ui);
}

fn upcoming(state: &State, scope: Option<Uid>, ui: &mut egui::Ui) {
    let p = &state.planner;
    let pal = palette_of(ui);
    let all = occurrences_all(&p.sessions, p.today, add_days(p.today, UPCOMING_DAYS));
    let list: Vec<_> = all
        .into_iter()
        .filter(|o| {
            scope.is_none_or(|u| {
                p.sessions
                    .iter()
                    .any(|s| s.id == o.session && s.stack == Some(u))
            })
        })
        .collect();
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
    home_hero::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            kit::pill(ui, event_kind_name(s.kind), kind_color(&pal, s.kind));
            ui.label(RichText::new(&s.title).strong());
        });
        ui.label(format!("{when}{stack}"));
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            if s.kind == EventKind::Study
                && ui
                    .button(format!("Start rehearsal for {}", s.title))
                    .on_hover_text("Starts a rehearsal of this stack now.")
                    .clicked()
            {
                plan(out, PlanMsg::Rehearse(s.id));
            }
            let memory = s.stack.filter(|_| s.kind == EventKind::Study);
            super::memory::entry::offer(ui, memory, &s.title, out);
            if ui
                .small_button(format!("Edit {}", s.title))
                .on_hover_text("Opens this for editing.")
                .clicked()
            {
                plan(out, PlanMsg::Edit(s.id));
            }
            if s.repeat != Repeat::Never
                && ui
                    .small_button(format!("Skip {} today", s.title))
                    .on_hover_text("Skips this occurrence of the session.")
                    .clicked()
            {
                plan(out, PlanMsg::Skip(s.id, o.date));
            }
            if ui
                .small_button(format!("Delete {}", s.title))
                .on_hover_text("Deletes this after you confirm.")
                .clicked()
            {
                plan(out, PlanMsg::Delete(s.id));
            }
        });
    });
    ui.add_space(6.0);
}
