//! Single-day hourly agenda view with review workload (STORY-0142).

use eframe::egui::{self, RichText, vec2};
use studyfrog_calendar::session::{EventKind, Occurrence, Repeat, Session, event_kind_name};
use studyfrog_core::civil::{format_date, format_minute};
use studyfrog_core::uid::Uid;

use crate::msg::Msg;
use crate::plan_msg::PlanMsg;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::calendar_grid::kind_color;
use crate::view::kit;

const START_HOUR: u16 = 6;
const END_HOUR: u16 = 22;

fn stack_title(state: &State, uid: Option<Uid>) -> Option<String> {
    let uid = uid?;
    let v = state.vault.as_ref()?;
    v.overview
        .stacks
        .iter()
        .find(|s| s.uid == uid)
        .map(|s| s.title.clone())
}

/// Draw the single-day schedule agenda.
pub fn draw(state: &State, scope: Option<Uid>, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let p = &state.planner;
    let _pal = palette_of(ui);

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("Agenda for {}", format_date(p.selected)))
                .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                .strong(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if kit::primary_button(ui, "+ Add session").clicked() {
                out.push(Msg::Plan(PlanMsg::New));
            }
        });
    });

    ui.add_space(8.0);

    // Reviews summary banner for this day
    let day_num = studyfrog_core::civil::date_to_days(p.selected);
    let day_reviews: Vec<_> = p.reviews.iter().filter(|r| r.day == day_num).collect();
    if !day_reviews.is_empty() {
        kit::card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Due Reviews:").strong());
                let total_due: usize = day_reviews.iter().map(|r| r.items.len()).sum();
                ui.label(format!(
                    "{total_due} items across {} stacks",
                    day_reviews.len()
                ));
            });
        });
        ui.add_space(6.0);
    }

    // Hourly agenda timeline
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let day_events: Vec<&Occurrence> = p
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

            for hour in START_HOUR..=END_HOUR {
                draw_day_hour_slot(state, hour, &day_events, ui, out);
            }
        });
}

fn draw_day_hour_slot(
    state: &State,
    hour: u16,
    events: &[&Occurrence],
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    let pal = palette_of(ui);
    let row_min = hour * 60;
    let row_max = row_min + 60;

    let slot_events: Vec<&&Occurrence> = events
        .iter()
        .filter(|o| o.start_minute >= row_min && o.start_minute < row_max)
        .collect();

    ui.horizontal_top(|ui| {
        ui.allocate_ui(vec2(60.0, 36.0), |ui| {
            ui.label(
                RichText::new(format!("{hour:02}:00"))
                    .monospace()
                    .color(pal.text_weak),
            );
        });

        ui.vertical(|ui| {
            if slot_events.is_empty() {
                ui.add_space(18.0);
                ui.separator();
            } else {
                for &occ in slot_events {
                    if let Some(session) =
                        state.planner.sessions.iter().find(|s| s.id == occ.session)
                    {
                        draw_day_card(state, session, occ, ui, out);
                    }
                }
            }
        });
    });
    ui.add_space(4.0);
}

fn draw_day_card(
    state: &State,
    s: &Session,
    o: &Occurrence,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    let pal = palette_of(ui);
    let end = o.start_minute + o.duration_min;
    let time_span = format!(
        "{}–{}",
        format_minute(o.start_minute),
        format_minute(end % 1440)
    );
    let stack_info =
        stack_title(state, s.stack).map_or(String::new(), |t| format!(" · Stack: {t}"));

    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            kit::pill(ui, event_kind_name(s.kind), kind_color(&pal, s.kind));
            ui.label(RichText::new(&s.title).strong());
            ui.label(
                RichText::new(format!(
                    "{time_span} ({duration}m)",
                    duration = o.duration_min
                ))
                .weak(),
            );
        });
        if !stack_info.is_empty() || !s.notes.is_empty() {
            ui.label(format!("{}{}", s.notes, stack_info));
        }
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            if s.kind == EventKind::Study
                && ui
                    .button(format!("Start rehearsal for {}", s.title))
                    .clicked()
            {
                out.push(Msg::Plan(PlanMsg::Rehearse(s.id)));
            }
            if ui.small_button("Edit").clicked() {
                out.push(Msg::Plan(PlanMsg::Edit(s.id)));
            }
            if s.repeat != Repeat::Never && ui.small_button("Skip today").clicked() {
                out.push(Msg::Plan(PlanMsg::Skip(s.id, o.date)));
            }
            if ui.small_button("Delete").clicked() {
                out.push(Msg::Plan(PlanMsg::Delete(s.id)));
            }
        });
    });
}
