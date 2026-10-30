//! The home screen: what is due now, what is coming up, and what happened recently. The three columns
//! stretch to the window so nothing important hides below the fold and no part sits empty.

use eframe::egui::{self, RichText};
use studyfrog_core::plan::item_kind_name;
use studyfrog_service::home::HomeSummary;

use crate::home_ui::HomeMsg;
use crate::msg::Msg;
use crate::state::{Screen, State};
use crate::theme::palette_of;
use crate::view::{home_cards, kit};

const DAY_MS: i64 = 86_400_000;
const MIN_COLUMN_HEIGHT: f32 = 240.0;

fn greeting(hour: u8) -> &'static str {
    match hour {
        5..=11 => "Good morning",
        12..=17 => "Good afternoon",
        _ => "Good evening",
    }
}

pub(super) fn ago(now_ms: i64, then_ms: i64) -> String {
    match (now_ms - then_ms).div_euclid(DAY_MS) {
        i64::MIN..=0 => "today".to_owned(),
        1 => "yesterday".to_owned(),
        n => format!("{n} days ago"),
    }
}

fn overdue(now_ms: i64, due_ms: i64) -> String {
    match (now_ms - due_ms).div_euclid(DAY_MS) {
        i64::MIN..=0 => "due today".to_owned(),
        1 => "1 day overdue".to_owned(),
        n => format!("{n} days overdue"),
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
    let viewport = ui.available_height();
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let top = ui.cursor().top();
            ui.add_space(4.0);
            ui.label(RichText::new(greeting(h.hour)).size(30.0).strong());
            let stacks = vault.overview.stacks.len();
            ui.label(
                RichText::new(format!("{} · {stacks} stacks", vault.name))
                    .color(palette_of(ui).text_weak),
            );
            ui.add_space(8.0);
            hero(
                h,
                vault
                    .overview
                    .stacks
                    .iter()
                    .map(|s| s.notes + s.cards + s.questions)
                    .sum(),
                ui,
                out,
            );
            ui.add_space(10.0);
            let fill = (viewport - (ui.cursor().top() - top) - 76.0).max(MIN_COLUMN_HEIGHT);
            ui.columns(3, |c| {
                due_column(h, fill, &mut c[0], out);
                home_cards::ahead_column(h, fill, &mut c[1]);
                home_cards::recent_column(h, fill, &mut c[2], out);
            });
        });
}

/// What is ready, the numbers behind the streak, and the one button that matters.
fn hero(h: &HomeSummary, items: usize, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            kit::stat(ui, &h.ready_today.to_string(), "items ready for today");
            ui.add_space(18.0);
            kit::stat(ui, &format!("{} days", h.streak_days), "Streak");
            ui.add_space(18.0);
            kit::stat(ui, &h.reviews_today.to_string(), "Reviews today");
            ui.add_space(18.0);
            kit::stat(ui, &items.to_string(), "Items in your vault");
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

pub(super) fn column_card(ui: &mut egui::Ui, fill: f32, add: impl FnOnce(&mut egui::Ui)) {
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.set_min_height(fill);
        add(ui);
    });
}

fn due_column(h: &HomeSummary, fill: f32, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    kit::section(ui, "Due now");
    column_card(ui, fill, |ui| {
        let p = palette_of(ui);
        ui.label(
            RichText::new(format!("{} due for review, {} new", h.due_now, h.new_items))
                .color(p.text_weak),
        );
        if h.due_items.is_empty() {
            ui.add_space(4.0);
            let text = match h.new_items {
                0 => "You are all caught up.".to_owned(),
                n => format!("Nothing is overdue. {n} new items are waiting to be learned."),
            };
            ui.label(text);
        }
        for i in &h.due_items {
            ui.horizontal(|ui| {
                kit::pill(ui, item_kind_name(i.kind), p.text_weak);
                if ui.link(&i.title).clicked() {
                    out.push(Msg::OpenItem(i.uid));
                }
                if let Some(due) = i.due {
                    ui.label(
                        RichText::new(overdue(h.now_ms, due.0))
                            .small()
                            .color(p.warning),
                    );
                }
            });
        }
        ui.add_space(8.0);
        ui.separator();
        stacks_waiting(h, ui, out);
    });
}

fn stacks_waiting(h: &HomeSummary, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let p = palette_of(ui);
    let mut waiting: Vec<_> = h.stacks.iter().filter(|s| s.due + s.new > 0).collect();
    waiting.sort_by_key(|s| std::cmp::Reverse(s.due * 1000 + s.new));
    if waiting.is_empty() {
        ui.weak("Nothing is waiting in any stack.");
    }
    for s in waiting.into_iter().take(6) {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(&s.title).strong());
            if s.due > 0 {
                kit::pill(ui, &format!("{} due", s.due), p.warning);
            }
            if s.new > 0 {
                kit::pill(ui, &format!("{} new", s.new), p.accent);
            }
            if ui.button(format!("Rehearse {}", s.title)).clicked() {
                out.push(Msg::Home(HomeMsg::RehearseStack(s.folder.clone())));
            }
        });
    }
}
