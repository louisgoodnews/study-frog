//! The home screen: what is due now, what is coming up, and what happened recently. The three columns
//! stretch to the window so nothing important hides below the fold and no part sits empty.
use crate::msg::MsgBrowse;

use eframe::egui::{self, RichText};
use studyfrog_core::plan::item_kind_name;
use studyfrog_service::home::HomeSummary;

use crate::home_ui::HomeMsg;
use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::{home_cards, home_hero, kit, library_pdfs};

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
            ui.label(
                RichText::new(greeting(h.hour))
                    .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                    .strong(),
            );
            if crate::tutorial::offered(state) {
                tour_card(ui, out);
            }
            let stacks = vault.overview.stacks.len();
            ui.label(RichText::new(format!("{stacks} stacks")).color(palette_of(ui).text_weak));
            ui.add_space(8.0);
            home_hero::draw(
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
            home_cards::attention_card(h, ui, out);
            ui.add_space(10.0);
            crate::view::diary::home_hint::draw(state, ui, out);
            ui.add_space(10.0);
            let fill = (viewport - (ui.cursor().top() - top) - 76.0).max(MIN_COLUMN_HEIGHT);
            ui.columns(3, |c| {
                due_column(h, fill, &mut c[0], out);
                home_cards::ahead_column(h, fill, &mut c[1]);
                home_cards::recent_column(h, fill, &mut c[2], out);
            });
            ui.add_space(10.0);
            library_pdfs::recent_card(state, ui, out);
        });
}

pub(super) fn column_card(ui: &mut egui::Ui, fill: f32, add: impl FnOnce(&mut egui::Ui)) {
    let pad = crate::theme_space::rhythm_of(ui).card;
    let inner = (ui.available_width() - 2.0 * pad).max(40.0);
    home_hero::card(ui, |ui| {
        ui.set_min_size(egui::vec2(inner, fill));
        ui.set_max_width(inner);
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
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
            ui.horizontal_wrapped(|ui| {
                kit::pill(ui, item_kind_name(i.kind), p.text_weak);
                if ui
                    .link(&i.title)
                    .on_hover_text("Opens this item.")
                    .clicked()
                {
                    out.push(Msg::Browse(MsgBrowse::OpenItem(i.uid)));
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
            let name = format!("Rehearse {}", s.title);
            let button = ui.button("Rehearse").on_hover_text("Starts a rehearsal.");
            button.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, &name));
            if button.clicked() {
                out.push(Msg::Home(HomeMsg::RehearseStack(s.folder.clone())));
            }
        });
        ui.add_space(4.0);
    }
}

/// The first-run tour offer: it stays until the user starts it or says not now.
fn tour_card(ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let pal = palette_of(ui);
    egui::Frame::new()
        .fill(pal.surface)
        .stroke(egui::Stroke::new(1.0, pal.border))
        .corner_radius(10)
        .inner_margin(14)
        .show(ui, |ui| {
            ui.label(RichText::new("New here? Take the first steps").strong());
            ui.label(
                "A short tour: a stack, a card, a rehearsal and the calendar. It stays in Help.",
            );
            ui.horizontal(|ui| {
                if kit::primary_button(ui, "Start the tour")
                    .on_hover_text("Starts the first-run tour, one step at a time.")
                    .clicked()
                {
                    out.push(Msg::Modal(crate::modal_types::ModalMsg::TourStart));
                }
                if kit::quiet_button(ui, "Not now")
                    .on_hover_text("Hides this card. The tour stays in Help.")
                    .clicked()
                {
                    out.push(Msg::Modal(crate::modal_types::ModalMsg::TourDecline));
                }
            });
        });
    ui.add_space(12.0);
}
