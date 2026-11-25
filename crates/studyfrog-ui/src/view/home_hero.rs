//! The Home headline and statistic ring in the redesign style (EPIC-070, STORY-0214). The same figures and the same
//! actions as before; only the drawing changes.

use eframe::egui::{self, Align2, FontId, RichText, Sense, Stroke};
use studyfrog_core::count::counted;
use studyfrog_service::home::HomeSummary;

use crate::home_ui::HomeMsg;
use crate::msg::{Msg, MsgVault};
use crate::state::Screen;
use crate::theme::palette_of;
use crate::theme::style::{CARD_RADIUS, style_of};
use crate::theme_space::rhythm_of;
use crate::view::{gradient_panel, home_ring, kit};

const HEADLINE_HEIGHT: f32 = 220.0;
const RING_SIZE: f32 = 150.0;
const RING_WIDTH: f32 = 10.0;

/// A card in the redesign style: the surface fill, the outline and the card radius.
pub(super) fn card(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    let s = style_of(&palette_of(ui));
    egui::Frame::new()
        .fill(s.card)
        .stroke(Stroke::new(1.0, s.card_edge))
        .corner_radius(CARD_RADIUS)
        .inner_margin(rhythm_of(ui).card.round() as i8)
        .show(ui, add);
}

/// The headline (left) and the statistic ring (right), sharing one row.
pub(super) fn draw(h: &HomeSummary, items: usize, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let total = ui.available_width();
    let gap = 12.0;
    let left = (total * 0.64 - gap / 2.0).max(200.0);
    let right = (total - left - gap).max(160.0);
    ui.vertical(|ui| {
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            ui.vertical(|ui| {
                ui.set_width(left);
                headline(h, ui, out);
            });
            ui.vertical(|ui| {
                ui.set_width(right);
                ring_card(h, right, ui);
            });
        });
        ui.add_space(gap);
        figures_card(h, items, total, ui);
    });
}

fn headline(h: &HomeSummary, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    gradient_panel::panel(ui, HEADLINE_HEIGHT, |ui, s| {
        ui.label(RichText::new("Up next").small().color(s.gradient_text));
        let title = h
            .due_items
            .first()
            .map_or("Nothing due right now".to_owned(), |i| i.title.clone());
        ui.label(
            RichText::new(title)
                .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                .strong()
                .color(s.gradient_text),
        );
        ui.label(
            RichText::new(format!("{} items ready for today", h.ready_today))
                .color(s.gradient_text),
        );
        ui.add_space(10.0);
        headline_action(h, s.gradient_text, s.gradient[0], ui, out);
    });
}

/// The one action of the headline, in the same words and with the same messages as before.
fn headline_action(
    h: &HomeSummary,
    fill: egui::Color32,
    text: egui::Color32,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    if h.ready_today > 0 {
        let button = egui::Button::new(RichText::new("Start today's rehearsal").color(text))
            .fill(fill)
            .min_size(egui::vec2(0.0, crate::theme::MIN_TARGET));
        if ui
            .add(button)
            .on_hover_text("Starts a rehearsal of what is due today.")
            .clicked()
        {
            out.push(Msg::Home(HomeMsg::StartToday));
        }
    } else {
        let button = egui::Button::new(RichText::new("Practice something else").color(text))
            .fill(fill)
            .min_size(egui::vec2(0.0, crate::theme::MIN_TARGET));
        if ui
            .add(button)
            .on_hover_text("Starts a different drill.")
            .clicked()
        {
            out.push(Msg::Vault(MsgVault::Navigate(Screen::Rehearse)));
        }
    }
}

/// The three figures that are not the ring, in one row.
fn figures_card(h: &HomeSummary, items: usize, width: f32, ui: &mut egui::Ui) {
    card(ui, |ui| {
        ui.set_width(width - 2.0 * rhythm_of(ui).card);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 40.0;
            kit::stat(ui, &h.ready_today.to_string(), "items ready for today");
            kit::stat(ui, &counted(h.streak_days, "day", "days"), "Streak");
            kit::stat(ui, &items.to_string(), "Items in your vault");
        });
    });
}

fn ring_card(h: &HomeSummary, width: f32, ui: &mut egui::Ui) {
    let p = palette_of(ui);
    let s = style_of(&p);
    card(ui, |ui| {
        ui.set_width(width - 2.0 * rhythm_of(ui).card);
        ui.vertical_centered(|ui| {
            let fraction = home_ring::fraction(
                h.reviews_today,
                h.reviews_today + u32::try_from(h.ready_today).unwrap_or(u32::MAX),
            );
            let (rect, _) =
                ui.allocate_exact_size(egui::vec2(RING_SIZE, RING_SIZE), Sense::hover());
            let centre = rect.center();
            let radius = RING_SIZE / 2.0 - RING_WIDTH;
            let number_size = crate::typo::size(ui, crate::typo::Tier::Heading);
            let painter = ui.painter();
            painter.circle_stroke(centre, radius, Stroke::new(RING_WIDTH, s.ring_track));
            painter.add(egui::Shape::line(
                home_ring::arc_points(centre, radius, fraction, 64),
                Stroke::new(RING_WIDTH, s.ring_fill),
            ));
            painter.text(
                centre,
                Align2::CENTER_CENTER,
                h.reviews_today.to_string(),
                FontId::proportional(number_size),
                p.text,
            );
            ui.label(RichText::new("Reviews today").color(p.text_weak));
        });
    });
}
