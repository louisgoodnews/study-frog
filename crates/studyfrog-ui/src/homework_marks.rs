//! Homework deadlines on the calendar (STORY-0166): urgency colours, day-cell dots and the day
//! card. The urgency rules live in `studyfrog_calendar::homework_query`; this file only paints.

use eframe::egui::{self, Align2, Color32, Painter, Pos2, Rect, RichText, Sense, vec2};
use studyfrog_calendar::homework::HomeworkItem;
use studyfrog_calendar::homework_query::{Urgency, due_on, urgency};
use studyfrog_core::civil::{Date, LocalTime, format_minute};
use studyfrog_core::uid::Uid;

use crate::planner::Planner;
use crate::theme::Palette;
use crate::typo;
use crate::view::kit;

/// Most dots drawn in one day cell; the rest are counted.
const MAX_DOTS: usize = 3;

/// The homework due on `day` in the calendar scope: every item, or only those of one course or
/// stack when `scope` is set.
pub fn due_for(p: &Planner, day: Date, scope: Option<Uid>) -> Vec<&HomeworkItem> {
    due_on(&p.homework, day)
        .into_iter()
        .filter(|i| scope.is_none_or(|u| i.course == Some(u)))
        .collect()
}

/// The colour that marks an urgency.
pub fn urgency_colour(pal: &Palette, u: Urgency) -> Color32 {
    match u {
        Urgency::Overdue => pal.danger,
        Urgency::DueToday => pal.warning,
        Urgency::Upcoming => pal.accent,
        Urgency::Later => pal.text_weak,
        Urgency::Completed => pal.success,
    }
}

/// The urgency as a word, for text and tooltips.
pub fn urgency_name(u: Urgency) -> &'static str {
    match u {
        Urgency::Overdue => "Overdue",
        Urgency::DueToday => "Due today",
        Urgency::Upcoming => "Due soon",
        Urgency::Later => "Later",
        Urgency::Completed => "Done",
    }
}

/// Dots along the bottom-right of a cell, one per item up to `MAX_DOTS`, then a "+n" count.
pub fn draw_dots(
    painter: &Painter,
    rect: Rect,
    items: &[&HomeworkItem],
    now: LocalTime,
    pal: &Palette,
) {
    for (i, item) in items.iter().take(MAX_DOTS).enumerate() {
        let at = Pos2::new(rect.right() - 8.0 - i as f32 * 10.0, rect.bottom() - 8.0);
        painter.circle_filled(at, 3.5, urgency_colour(pal, urgency(item, now)));
    }
    if items.len() > MAX_DOTS {
        let at = Pos2::new(
            rect.right() - 8.0 - MAX_DOTS as f32 * 10.0,
            rect.bottom() - 8.0,
        );
        painter.text(
            at,
            Align2::RIGHT_CENTER,
            format!("+{}", items.len() - MAX_DOTS),
            typo::painter_font(painter, typo::Tier::Caption),
            pal.text_weak,
        );
    }
}

/// One line per item for a hover tooltip: title, due time and urgency.
pub fn tooltip(items: &[&HomeworkItem], now: LocalTime) -> String {
    items
        .iter()
        .map(|i| {
            format!(
                "{} · due {} · {}",
                i.title,
                format_minute(i.due.minute),
                urgency_name(urgency(i, now))
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The day view's list of homework due that day, most pressing text first as the caller orders it.
pub fn day_card(ui: &mut egui::Ui, items: &[&HomeworkItem], now: LocalTime, pal: &Palette) {
    if items.is_empty() {
        return;
    }
    kit::card(ui, |ui| {
        ui.label(RichText::new("Homework due:").strong());
        for item in items {
            let u = urgency(item, now);
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
                ui.painter()
                    .circle_filled(rect.center(), 4.0, urgency_colour(pal, u));
                ui.label(format!(
                    "{} · due {} · {}",
                    item.title,
                    format_minute(item.due.minute),
                    urgency_name(u)
                ));
            });
        }
    });
    ui.add_space(6.0);
}
