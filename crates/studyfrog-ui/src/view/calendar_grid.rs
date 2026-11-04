//! The month grid of the calendar: a header, six weeks of day cells with the entries as coloured chips.

use eframe::egui::{
    self, Align2, Color32, FontId, Rect, RichText, Role, Sense, Stroke, WidgetInfo, vec2,
};
use studyfrog_calendar::session::{EventKind, Occurrence, Session};
use studyfrog_core::civil::{Date, format_date};
use studyfrog_core::uid::Uid;

use crate::msg::Msg;
use crate::planner::{PlanMsg, grid_of};
use crate::state::State;
use crate::theme::{Palette, palette_of, tint};
use crate::view::calendar_reviews;

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
const CHIPS: usize = 3;

fn plan(out: &mut Vec<Msg>, m: PlanMsg) {
    out.push(Msg::Plan(m));
}

/// The colour an entry kind is drawn in.
pub fn kind_color(p: &Palette, kind: EventKind) -> Color32 {
    match kind {
        EventKind::Study => p.accent,
        EventKind::Task => p.success,
        EventKind::Appointment => p.alt,
        EventKind::Exam => p.danger,
        EventKind::Deadline => p.warning,
    }
}

pub fn month(state: &State, scope: Option<Uid>, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let p = &state.planner;
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("Calendar").size(17.0).strong());
        ui.add_space(12.0);
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
    ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
    let width = ((ui.available_width() - 6.0 * 4.0) / 7.0).max(44.0);
    let height = ((ui.available_height() - 24.0 - 6.0 * 4.0) / 6.0).clamp(64.0, 160.0);
    ui.horizontal(|ui| {
        for name in DAY_NAMES {
            ui.add_sized(
                [width, 18.0],
                egui::Label::new(RichText::new(name).strong()),
            );
        }
    });
    let (mut cells, mut dropped) = (Vec::new(), Vec::new());
    for week in grid_of(p.month).chunks(7) {
        ui.horizontal(|ui| {
            for day in week {
                let size = vec2(width, height);
                let (rect, more) = day_cell(state, (*day, scope), size, (ui, out));
                cells.push((*day, rect));
                dropped.extend(more);
            }
        });
    }
    calendar_reviews::resolve(&dropped, &cells, out);
}

fn day_cell(
    state: &State,
    (day, scope): (Date, Option<Uid>),
    size: egui::Vec2,
    (ui, out): (&mut egui::Ui, &mut Vec<Msg>),
) -> (Rect, Vec<calendar_reviews::Dropped>) {
    let p = &state.planner;
    let pal = palette_of(ui);
    let entries: Vec<(&Occurrence, &Session)> = p
        .shown
        .iter()
        .filter(|o| o.date == day)
        .filter_map(|o| {
            p.sessions
                .iter()
                .find(|s| s.id == o.session)
                .map(|s| (o, s))
        })
        .filter(|(_, s)| scope.is_none_or(|u| s.stack == Some(u)))
        .collect();
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let name = if entries.is_empty() {
        day.day.to_string()
    } else {
        format!("{} · {}", day.day, entries.len())
    };
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, name.clone()));
    let selected = day == p.selected;
    let fill = if selected {
        tint(pal.accent, 40)
    } else if response.hovered() {
        pal.surface_hover
    } else {
        pal.surface
    };
    let edge = if day == p.today {
        pal.accent
    } else {
        pal.border
    };
    let painter = ui.painter_at(rect);
    painter.rect(
        rect,
        6.0,
        fill,
        Stroke::new(if day == p.today { 1.5 } else { 1.0 }, edge),
        egui::StrokeKind::Inside,
    );
    let in_month = (day.year, day.month) == p.month;
    let ink = if in_month { pal.text } else { pal.text_weak };
    painter.text(
        rect.left_top() + vec2(6.0, 4.0),
        Align2::LEFT_TOP,
        day.day.to_string(),
        FontId::proportional(14.0),
        ink,
    );
    let used = chips(&painter, rect, &entries, &pal);
    let reviews = calendar_reviews::day_chips(state, day, (rect, used), scope, (ui, out));
    if response.on_hover_text(format_date(day)).clicked() {
        plan(out, PlanMsg::Select(day));
    }
    (rect, reviews)
}

/// Draws the session chips and returns how many lines they took.
fn chips(
    painter: &egui::Painter,
    rect: Rect,
    entries: &[(&Occurrence, &Session)],
    pal: &Palette,
) -> usize {
    for (i, (_, s)) in entries.iter().take(CHIPS).enumerate() {
        let top = rect.top() + 24.0 + i as f32 * 17.0;
        let chip = Rect::from_min_size(
            egui::pos2(rect.left() + 4.0, top),
            vec2(rect.width() - 8.0, 15.0),
        );
        if chip.bottom() > rect.bottom() {
            return i;
        }
        let color = kind_color(pal, s.kind);
        painter.rect_filled(chip, 3.0, tint(color, 60));
        let clipped = painter.with_clip_rect(chip);
        clipped.text(
            chip.left_center() + vec2(4.0, 0.0),
            Align2::LEFT_CENTER,
            &s.title,
            FontId::proportional(11.5),
            pal.text,
        );
    }
    if entries.len() > CHIPS {
        let more = format!("+{} more", entries.len() - CHIPS);
        painter.text(
            rect.right_bottom() - vec2(6.0, 3.0),
            Align2::RIGHT_BOTTOM,
            more,
            FontId::proportional(11.0),
            pal.text_weak,
        );
    }
    entries.len().min(CHIPS)
}
