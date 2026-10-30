//! The home screen: what is ready today, the streak, what is planned, which stacks are waiting, and
//! what happened recently.

use eframe::egui::{self, RichText};
use studyfrog_core::civil::format_minute;
use studyfrog_core::plan::item_kind_name;
use studyfrog_service::home::{HomeSummary, StackDue};

use crate::home_ui::HomeMsg;
use crate::msg::Msg;
use crate::state::{Screen, State};
use crate::theme::palette_of;
use crate::view::kit;

const DAY_MS: i64 = 86_400_000;

fn greeting(hour: u8) -> &'static str {
    match hour {
        5..=11 => "Good morning",
        12..=17 => "Good afternoon",
        _ => "Good evening",
    }
}

fn ago(now_ms: i64, then_ms: i64) -> String {
    match (now_ms - then_ms).div_euclid(DAY_MS) {
        i64::MIN..=0 => "today".to_owned(),
        1 => "yesterday".to_owned(),
        n => format!("{n} days ago"),
    }
}

/// The whole page.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(vault) = &state.vault else { return };
    let Some(h) = &state.home else {
        ui.heading("Home");
        ui.label("Looking at what is ready…");
        return;
    };
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(4.0);
        ui.label(RichText::new(greeting(h.hour)).size(30.0).strong());
        ui.label(
            RichText::new(format!(
                "{} · {} stacks",
                vault.name,
                vault.overview.stacks.len()
            ))
            .color(palette_of(ui).text_weak),
        );
        ui.add_space(10.0);
        hero(h, ui, out);
        ui.add_space(10.0);
        let items: usize = vault
            .overview
            .stacks
            .iter()
            .map(|s| s.notes + s.cards + s.questions)
            .sum();
        ui.columns(3, |c| {
            for (i, (value, caption)) in [
                (format!("{} days", h.streak_days), "Streak"),
                (h.reviews_today.to_string(), "Reviews today"),
                (items.to_string(), "Items in your vault"),
            ]
            .into_iter()
            .enumerate()
            {
                kit::card(&mut c[i], |ui| {
                    ui.set_width(ui.available_width());
                    kit::stat(ui, &value, caption)
                });
            }
        });
        ui.add_space(10.0);
        ui.columns(2, |c| {
            stacks_card(h, &mut c[0], out);
            planned_card(h, &mut c[0]);
            recent_items_card(h, &mut c[1], out);
            recent_runs_card(h, &mut c[1]);
        });
    });
}

fn hero(h: &HomeSummary, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    kit::card(ui, |ui| {
        ui.horizontal(|ui| {
            kit::stat(ui, &h.ready_today.to_string(), "items ready for today");
            ui.vertical(|ui| {
                ui.add_space(8.0);
                ui.label(format!("{} due for review, {} new", h.due_now, h.new_items));
                if h.ready_today == 0 {
                    ui.weak("You are all caught up. Practice anything from the Rehearse screen.");
                }
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if h.ready_today > 0 {
                    if kit::primary_button(ui, "Start today's rehearsal").clicked() {
                        out.push(Msg::Home(HomeMsg::StartToday));
                    }
                } else if ui.button("Practice something else").clicked() {
                    out.push(Msg::Navigate(Screen::Rehearse));
                }
            });
        });
    });
}

fn stacks_card(h: &HomeSummary, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    kit::section(ui, "Stacks");
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        let mut waiting: Vec<&StackDue> = h.stacks.iter().filter(|s| s.due + s.new > 0).collect();
        waiting.sort_by_key(|s| std::cmp::Reverse(s.due * 1000 + s.new));
        if waiting.is_empty() {
            ui.weak("Nothing is waiting in any stack.");
        }
        for s in waiting.into_iter().take(6) {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&s.title).strong());
                let p = palette_of(ui);
                if s.due > 0 {
                    kit::pill(ui, &format!("{} due", s.due), p.warning);
                }
                if s.new > 0 {
                    kit::pill(ui, &format!("{} new", s.new), p.accent);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(format!("Rehearse {}", s.title)).clicked() {
                        out.push(Msg::Home(HomeMsg::RehearseStack(s.folder.clone())));
                    }
                });
            });
        }
    });
}

fn planned_card(h: &HomeSummary, ui: &mut egui::Ui) {
    kit::section(ui, "Planned today");
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
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
    });
}

fn recent_items_card(h: &HomeSummary, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    kit::section(ui, "Recently edited");
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        if h.recent_items.is_empty() {
            ui.weak("No items yet. Use Create to make your first one.");
        }
        for i in &h.recent_items {
            ui.horizontal(|ui| {
                kit::pill(ui, item_kind_name(i.kind), palette_of(ui).text_weak);
                if ui.link(&i.title).clicked() {
                    out.push(Msg::OpenItem(i.uid));
                }
            });
        }
    });
}

fn recent_runs_card(h: &HomeSummary, ui: &mut egui::Ui) {
    kit::section(ui, "Recent runs");
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
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
