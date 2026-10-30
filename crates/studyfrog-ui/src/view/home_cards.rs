//! The second and third column of the home screen: what is coming up and what happened recently.

use eframe::egui::{self, RichText};
use studyfrog_calendar::session::event_kind_name;
use studyfrog_core::civil::{format_date, format_minute};
use studyfrog_core::plan::item_kind_name;
use studyfrog_service::home::HomeSummary;

use crate::msg::Msg;
use crate::theme::palette_of;
use crate::view::calendar_grid::kind_color;
use crate::view::home::{ago, column_card};
use crate::view::kit;

pub(super) fn ahead_column(h: &HomeSummary, fill: f32, ui: &mut egui::Ui) {
    kit::section(ui, "Coming up");
    column_card(ui, fill, |ui| {
        let p = palette_of(ui);
        ui.label(RichText::new("Today").strong());
        if h.sessions_today.is_empty() {
            ui.weak("No study sessions today. Plan one in the Calendar.");
        }
        for s in &h.sessions_today {
            let end = (s.start_minute + s.duration_min) % 1440;
            let stack = s
                .stack
                .as_deref()
                .map_or(String::new(), |t| format!(" · {t}"));
            ui.label(format!(
                "{}–{}  {}{stack}",
                format_minute(s.start_minute),
                format_minute(end),
                s.title
            ));
        }
        ui.add_space(8.0);
        ui.label(RichText::new("Next 7 days").strong());
        if h.upcoming.is_empty() {
            ui.weak("Nothing planned. Exams, tasks and deadlines you add show up here.");
        }
        for u in &h.upcoming {
            ui.horizontal(|ui| {
                kit::pill(ui, event_kind_name(u.kind), kind_color(&p, u.kind));
                ui.label(format!(
                    "{} {}  {}",
                    format_date(u.date),
                    format_minute(u.start_minute),
                    u.title
                ));
            });
        }
    });
}

pub(super) fn recent_column(h: &HomeSummary, fill: f32, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    kit::section(ui, "Recent activity");
    column_card(ui, fill, |ui| {
        let p = palette_of(ui);
        ui.label(RichText::new("Recently edited").strong());
        if h.recent_items.is_empty() {
            ui.weak("No items yet. Use Create to make your first one.");
        }
        for i in &h.recent_items {
            ui.horizontal(|ui| {
                kit::pill(ui, item_kind_name(i.kind), p.text_weak);
                if ui.link(&i.title).clicked() {
                    out.push(Msg::OpenItem(i.uid));
                }
            });
        }
        ui.add_space(8.0);
        ui.label(RichText::new("Recent runs").strong());
        if h.recent_runs.is_empty() {
            ui.weak("You have not rehearsed yet.");
        }
        for r in &h.recent_runs {
            let how = if r.state == "finished" {
                "finished"
            } else {
                "stopped early"
            };
            let mode = if r.mode == "answer" {
                "typed answers"
            } else {
                "flip cards"
            };
            ui.label(format!(
                "{}: {} rated, {} skipped, {how} ({mode})",
                ago(h.now_ms, r.started_ms),
                r.rated,
                r.skipped
            ));
        }
    });
}
