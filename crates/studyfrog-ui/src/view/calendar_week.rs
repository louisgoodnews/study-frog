//! Calendar 7-day weekly schedule view (STORY-0142).

use eframe::egui::{self, RichText, Sense, vec2};
use studyfrog_calendar::recurrence::occurrences_all;
use studyfrog_calendar::session::{Occurrence, Session, event_kind_name};
use studyfrog_core::civil::{Date, add_days, format_date, format_minute, weekday, weekday_index};
use studyfrog_core::uid::Uid;

use crate::msg::Msg;
use crate::plan_msg::PlanMsg;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::calendar_grid::kind_color;
use crate::view::kit;

const START_HOUR: u16 = 6;
const END_HOUR: u16 = 22;
const ROW_HEIGHT: f32 = 42.0;
const TIME_COL_WIDTH: f32 = 50.0;

fn week_days(selected: Date) -> [Date; 7] {
    let offset = i64::from(weekday_index(weekday(selected)));
    let monday = add_days(selected, -offset);
    std::array::from_fn(|i| add_days(monday, i as i64))
}

fn weekday_name(idx: usize) -> &'static str {
    match idx {
        0 => "Mon",
        1 => "Tue",
        2 => "Wed",
        3 => "Thu",
        4 => "Fri",
        5 => "Sat",
        _ => "Sun",
    }
}

/// Draw the 7-day hourly grid.
pub fn draw(state: &State, scope: Option<Uid>, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let p = &state.planner;
    let pal = palette_of(ui);
    let days = week_days(p.selected);
    let occurrences = occurrences_all(&p.sessions, days[0], days[6]);

    let avail_width = ui.available_width();
    let col_width = ((avail_width - TIME_COL_WIDTH - 16.0) / 7.0).max(100.0);

    // Week header row
    ui.horizontal(|ui| {
        ui.allocate_space(vec2(TIME_COL_WIDTH, 26.0));
        for (i, &day) in days.iter().enumerate() {
            let is_today = day == p.today;
            let is_selected = day == p.selected;
            let (rect, resp) = ui.allocate_exact_size(vec2(col_width, 28.0), Sense::click());
            if resp.clicked() {
                out.push(Msg::Plan(PlanMsg::Select(day)));
            }
            if is_selected {
                ui.painter()
                    .rect_filled(rect, 4.0, pal.accent.gamma_multiply(0.18));
            }
            let label = format!("{} {}", weekday_name(i), day.day);
            let mut text = RichText::new(label).strong();
            if is_today {
                text = text.color(pal.accent);
            }
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                text.text(),
                crate::typo::font(ui, crate::typo::Tier::Label),
                if is_today { pal.accent } else { pal.text },
            );
        }
    });

    ui.separator();

    // Scrollable hourly schedule area
    let params = WeekRowParams {
        state,
        scope,
        days: &days,
        col_width,
        occurrences: &occurrences,
    };
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for hour in START_HOUR..=END_HOUR {
                draw_hour_row(&params, hour, ui, out);
            }
        });
}

struct WeekRowParams<'a> {
    state: &'a State,
    scope: Option<Uid>,
    days: &'a [Date; 7],
    col_width: f32,
    occurrences: &'a [Occurrence],
}

fn draw_hour_row(params: &WeekRowParams<'_>, hour: u16, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let pal = palette_of(ui);
    let row_min = hour * 60;
    let row_max = row_min + 60;

    ui.horizontal(|ui| {
        // Time label column (e.g. 09:00)
        let (t_rect, _) = ui.allocate_exact_size(vec2(TIME_COL_WIDTH, ROW_HEIGHT), Sense::hover());
        ui.painter().text(
            t_rect.center_top(),
            egui::Align2::CENTER_TOP,
            format!("{hour:02}:00"),
            egui::FontId::monospace(crate::typo::size(ui, crate::typo::Tier::Caption)),
            pal.text_weak,
        );

        // 7 day slots
        for &day in params.days {
            let (rect, resp) =
                ui.allocate_exact_size(vec2(params.col_width, ROW_HEIGHT), Sense::click());
            if resp.clicked() {
                out.push(Msg::Plan(PlanMsg::Select(day)));
            }
            ui.painter().rect_stroke(
                rect,
                0.0,
                egui::Stroke::new(0.5, pal.border.gamma_multiply(0.3)),
                egui::StrokeKind::Inside,
            );

            // Filter occurrences for this day & hour slot
            let slot_events: Vec<&Occurrence> = params
                .occurrences
                .iter()
                .filter(|o| o.date == day && o.start_minute >= row_min && o.start_minute < row_max)
                .filter(|o| {
                    params.scope.is_none_or(|u| {
                        params
                            .state
                            .planner
                            .sessions
                            .iter()
                            .any(|s| s.id == o.session && s.stack == Some(u))
                    })
                })
                .collect();

            for (idx, &occ) in slot_events.iter().enumerate() {
                if let Some(session) = params
                    .state
                    .planner
                    .sessions
                    .iter()
                    .find(|s| s.id == occ.session)
                {
                    draw_event_chip(rect, idx, session, occ, ui, out);
                }
            }
        }
    });
}

fn draw_event_chip(
    cell: egui::Rect,
    idx: usize,
    s: &Session,
    occ: &Occurrence,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    let pal = palette_of(ui);
    let offset_y = (idx as f32) * 20.0;
    if offset_y + 18.0 > cell.height() {
        return;
    }
    let chip_rect = egui::Rect::from_min_size(
        egui::pos2(cell.min.x + 2.0, cell.min.y + 2.0 + offset_y),
        vec2(cell.width() - 4.0, 18.0),
    );

    let base_color = kind_color(&pal, s.kind);
    ui.painter()
        .rect_filled(chip_rect, 3.0, base_color.gamma_multiply(0.22));
    ui.painter().rect_stroke(
        chip_rect,
        3.0,
        egui::Stroke::new(1.0, base_color),
        egui::StrokeKind::Inside,
    );

    let time_str = format_minute(occ.start_minute);
    let text = format!("{time_str} {}", s.title);
    let resp = ui.allocate_rect(chip_rect, Sense::click());
    if resp.clicked() {
        out.push(Msg::Plan(PlanMsg::Select(occ.date)));
        out.push(Msg::Plan(PlanMsg::Edit(s.id)));
    }
    resp.on_hover_ui(|ui| {
        kit::pill(ui, event_kind_name(s.kind), base_color);
        ui.label(RichText::new(&s.title).strong());
        ui.label(format!(
            "{} · {} min",
            format_date(occ.date),
            occ.duration_min
        ));
        if !s.notes.is_empty() {
            ui.label(&s.notes);
        }
    });

    let text_rect = chip_rect.shrink2(vec2(3.0, 1.0));
    ui.painter().text(
        text_rect.left_center(),
        egui::Align2::LEFT_CENTER,
        text,
        crate::typo::font(ui, crate::typo::Tier::Caption),
        pal.text,
    );
}
