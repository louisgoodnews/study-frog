//! Cards, captions and section headings (STORY-0115): depth comes from tonal surface steps, not from outlines. A card
//! is the `surface` step on the `panel` page with no border; a card inside a card is an inset of the `panel` step; only
//! high contrast draws borders. Spacing follows the density's rhythm (`theme_space`): siblings `item` apart, `card`
//! padding inside, groups `section` apart, a caption `tight` above its card.

use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, RichText, Stroke};

use crate::theme::{look_of, palette_of};
use crate::theme_space::{in_card, rhythm_of};

/// The fill and stroke of a card at a nesting depth, given the fill and border asked for. The border is drawn only
/// with high contrast or when it carries meaning (an item's colour differs from the theme's border).
fn look_at(ui: &egui::Ui, depth: usize, (fill, border): (Color32, Color32)) -> (Color32, Stroke) {
    let p = palette_of(ui);
    let fill = if depth > 0 && fill == p.surface {
        p.panel
    } else {
        fill
    };
    let outline = look_of(ui.ctx()).high_contrast || border != p.border;
    (
        fill,
        if outline {
            Stroke::new(1.0, border)
        } else {
            Stroke::NONE
        },
    )
}

fn frame(ui: &egui::Ui, fill: Color32, stroke: Stroke) -> Frame {
    let pad = rhythm_of(ui).card;
    Frame::new()
        .fill(fill)
        .stroke(stroke)
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(pad.round() as i8))
}

/// A surface with rounded corners that holds a group of content.
pub fn card<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let p = palette_of(ui);
    card_with(ui, (p.surface, p.border), add)
}

/// A card with its own fill and border colour (an item's tint, see `item_look`).
pub fn card_with<R>(
    ui: &mut egui::Ui,
    colors: (Color32, Color32),
    add: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let ctx = ui.ctx().clone();
    in_card(&ctx, |depth| {
        let (fill, stroke) = look_at(ui, depth, colors);
        frame(ui, fill, stroke)
            .show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Min), add)
                    .inner
            })
            .inner
    })
}

/// A card that can be clicked as a whole; returns whether it was. Hovering fills it one step lighter and outlines it
/// in the accent; neither changes its size.
pub fn card_button(ui: &mut egui::Ui, id: &str, add: impl FnOnce(&mut egui::Ui)) -> bool {
    let p = palette_of(ui);
    let outer = ui.make_persistent_id(id);
    let (fill, stroke) = look_at(ui, 0, (p.surface, p.border));
    let mut prepared = frame(ui, fill, stroke).begin(ui);
    let width = prepared.content_ui.available_width();
    prepared.content_ui.set_width(width);
    prepared
        .content_ui
        .with_layout(egui::Layout::top_down(egui::Align::Min), add);
    let response = prepared.allocate_space(ui);
    let response = ui.interact(response.rect, outer, egui::Sense::click());
    if response.hovered() {
        prepared.frame.fill = p.surface_hover;
        prepared.frame.stroke = Stroke::new(1.0, p.accent);
    }
    prepared.paint(ui);
    response.clicked()
}

/// A heading for a block of a screen, a section's gap below what came before.
pub fn section(ui: &mut egui::Ui, title: &str) {
    let r = rhythm_of(ui);
    ui.add_space((r.section - ui.spacing().item_spacing.y).max(0.0));
    let size = crate::typo::size(ui, crate::typo::Tier::Heading);
    ui.label(RichText::new(title).size(size).strong());
}

/// A small grey caption above a group of controls: a section's gap above it, a tight gap below it.
pub fn caption(ui: &mut egui::Ui, text: &str) {
    let r = rhythm_of(ui);
    let item = ui.spacing().item_spacing.y;
    ui.add_space((r.section - item).max(0.0));
    ui.spacing_mut().item_spacing.y = r.tight;
    ui.label(
        RichText::new(text)
            .size(crate::typo::size(ui, crate::typo::Tier::Label))
            .strong()
            .color(palette_of(ui).text_weak),
    );
    ui.spacing_mut().item_spacing.y = item;
}

/// A caption and, under it, a card that holds the group.
pub fn group<R>(ui: &mut egui::Ui, title: &str, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    caption(ui, title);
    card(ui, |ui| {
        ui.set_width(ui.available_width());
        add(ui)
    })
}
