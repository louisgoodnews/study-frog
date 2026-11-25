//! Match mode in the redesign style (EPIC-073, STORY-0237): the gradient headline, the HUD row, and the 3x4
//! interactive tile grid. The victory summary is in `match_victory`. The same controls, messages and matching
//! rules as before; only the drawing changes.
//!
//! Complies with Invariant P1: zero inherent impl blocks; all logic in free functions.

use eframe::egui::{self, Color32, CornerRadius, RichText, Sense, Stroke, Vec2};
use studyfrog_rehearsal::match_mode::{MatchModeSession, MatchTile, TileStatus, format_match_time};

use crate::match_ui::{MatchMsg, default_match_pairs};
use crate::msg::Msg;
use crate::state::State;
use crate::theme::style::{CARD_RADIUS, Style, style_of};
use crate::theme::{Palette, palette_of};
use crate::view::{gradient_panel, match_victory};

/// The height of the headline panel, in points.
const HEADER_HEIGHT: f32 = 72.0;

/// The gradient headline: the screen's title on the headline panel.
fn headline(ui: &mut egui::Ui) {
    gradient_panel::panel(ui, HEADER_HEIGHT, |ui, s| {
        ui.label(
            RichText::new("Match")
                .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                .strong()
                .color(s.gradient_text),
        );
    });
}

fn draw_hud(session: &MatchModeSession, ui: &mut egui::Ui, out: &mut Vec<Msg>, p: &Palette) {
    ui.horizontal(|ui| {
        let total = session.tiles.len() / 2;
        let matched = total.saturating_sub(session.pairs_remaining);
        ui.label(
            RichText::new(format!("Pairs: {matched}/{total}"))
                .size(crate::typo::size(ui, crate::typo::Tier::Label))
                .strong(),
        );

        ui.with_layout(
            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
            |ui| {
                ui.label(
                    RichText::new(format_match_time(session.elapsed_ms))
                        .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                        .strong(),
                );
            },
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Exit").on_hover_text("Quits the app.").clicked() {
                out.push(Msg::Match(MatchMsg::Leave));
            }
            if ui
                .button("Restart")
                .on_hover_text("Starts again.")
                .clicked()
            {
                out.push(Msg::Match(MatchMsg::Restart));
            }
            let streak_color = if session.streak > 1 {
                p.accent
            } else {
                p.text_weak
            };
            ui.label(
                RichText::new(format!("Streak: {}x", session.streak))
                    .size(crate::typo::size(ui, crate::typo::Tier::Label))
                    .color(streak_color)
                    .strong(),
            );
        });
    });
}

/// The fill, outline and text colour of a tile in `status`. An idle tile is a redesign card.
fn tile_visuals(status: TileStatus, p: &Palette, s: &Style) -> (Color32, Stroke, Color32) {
    match status {
        TileStatus::Idle => (s.card, Stroke::new(1.0, s.card_edge), p.text),
        TileStatus::Selected => (
            crate::theme::tint(p.accent, 40),
            Stroke::new(2.5, p.accent),
            p.accent,
        ),
        TileStatus::Mismatch { .. } => (
            crate::theme::tint(p.danger, 40),
            Stroke::new(2.5, p.danger),
            p.danger,
        ),
        TileStatus::Matched => (Color32::TRANSPARENT, Stroke::NONE, Color32::TRANSPARENT),
    }
}

fn draw_tile(
    tile: &MatchTile,
    size: Vec2,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
    p: &Palette,
    s: &Style,
) {
    if tile.status == TileStatus::Matched {
        ui.allocate_exact_size(size, Sense::hover());
        return;
    }

    let (bg, stroke, text_color) = tile_visuals(tile.status, p, s);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());

    let fill = if resp.hovered() && tile.status == TileStatus::Idle {
        p.surface_hover
    } else {
        bg
    };

    let radius = CornerRadius::same(CARD_RADIUS);
    ui.painter().rect_filled(rect, radius, fill);
    ui.painter()
        .rect_stroke(rect, radius, stroke, egui::StrokeKind::Inside);

    let text = RichText::new(&tile.text)
        .size(crate::typo::size(ui, crate::typo::Tier::Body))
        .color(text_color);

    let wrap_width = (size.x - 16.0).max(10.0);
    let galley = ui.painter().layout(
        text.text().to_string(),
        egui::FontId::proportional(crate::typo::size(ui, crate::typo::Tier::Body)),
        text_color,
        wrap_width,
    );

    let text_pos = rect.center() - galley.size() * 0.5;
    ui.painter().galley(text_pos, galley, text_color);

    if resp.clicked() {
        out.push(Msg::Match(MatchMsg::SelectTile(tile.id)));
    }
}

fn draw_tile_grid(
    session: &MatchModeSession,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
    p: &Palette,
    s: &Style,
) {
    let avail_width = ui.available_width();
    let col_width = ((avail_width - 32.0) / 3.0).clamp(90.0, 240.0);
    let row_height = 80.0;
    let tile_size = Vec2::new(col_width, row_height);

    for row in 0..4 {
        ui.horizontal(|ui| {
            for col in 0..3 {
                let idx = row * 3 + col;
                if let Some(tile) = session.tiles.get(idx) {
                    draw_tile(tile, tile_size, ui, out, p, s);
                }
            }
        });
        ui.add_space(8.0);
    }
}

/// Draws the full match mode screen: the gradient headline, the HUD header, then the 3x4 grid or the victory
/// summary.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let match_state = match state.match_state.as_ref() {
        Some(ms) => ms,
        None => {
            out.push(Msg::Match(MatchMsg::Start(default_match_pairs())));
            return;
        }
    };

    let session = &match_state.session;
    let pal = palette_of(ui);
    let s = style_of(&pal);

    ui.ctx().request_repaint();

    headline(ui);
    ui.add_space(8.0);
    draw_hud(session, ui, out, &pal);
    ui.add_space(16.0);

    if session.completed {
        match_victory::draw(session, match_state.high_score, ui, out, &pal);
    } else {
        draw_tile_grid(session, ui, out, &pal, &s);
    }
}
