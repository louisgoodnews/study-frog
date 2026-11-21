//! 3x4 interactive match mode tile grid view and victory summary.
//!
//! Complies with Invariant P1: zero inherent impl blocks; all logic in free functions.

use eframe::egui::{self, Color32, CornerRadius, RichText, Sense, Stroke, Vec2};
use studyfrog_rehearsal::match_mode::{
    MatchModeSession, MatchTile, TileStatus, compute_match_score, format_match_time,
};

use crate::match_ui::{MatchMsg, default_match_pairs};
use crate::msg::Msg;
use crate::state::State;
use crate::theme::{Palette, palette_of};

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

fn tile_visuals(status: TileStatus, p: &Palette) -> (Color32, Stroke, Color32) {
    match status {
        TileStatus::Idle => (p.surface, Stroke::new(1.0, p.border), p.text),
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

fn draw_tile(tile: &MatchTile, size: Vec2, ui: &mut egui::Ui, out: &mut Vec<Msg>, p: &Palette) {
    if tile.status == TileStatus::Matched {
        ui.allocate_exact_size(size, Sense::hover());
        return;
    }

    let (bg, stroke, text_color) = tile_visuals(tile.status, p);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());

    let fill = if resp.hovered() && tile.status == TileStatus::Idle {
        p.surface_hover
    } else {
        bg
    };

    ui.painter().rect_filled(rect, 8.0, fill);
    ui.painter()
        .rect_stroke(rect, 8.0, stroke, egui::StrokeKind::Inside);

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

fn draw_tile_grid(session: &MatchModeSession, ui: &mut egui::Ui, out: &mut Vec<Msg>, p: &Palette) {
    let avail_width = ui.available_width();
    let col_width = ((avail_width - 32.0) / 3.0).clamp(90.0, 240.0);
    let row_height = 80.0;
    let tile_size = Vec2::new(col_width, row_height);

    for row in 0..4 {
        ui.horizontal(|ui| {
            for col in 0..3 {
                let idx = row * 3 + col;
                if let Some(tile) = session.tiles.get(idx) {
                    draw_tile(tile, tile_size, ui, out, p);
                }
            }
        });
        ui.add_space(8.0);
    }
}

fn draw_victory_screen(
    session: &MatchModeSession,
    high_score: u32,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
    p: &Palette,
) {
    ui.vertical_centered(|ui| {
        ui.add_space(20.0);
        ui.label(
            RichText::new("Match Complete!")
                .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                .color(p.success)
                .strong(),
        );
        ui.add_space(8.0);

        let score = compute_match_score(session);
        egui::Frame::new()
            .fill(p.surface)
            .stroke(Stroke::new(1.5, p.border))
            .corner_radius(CornerRadius::same(12))
            .inner_margin(egui::Margin::symmetric(32, 24))
            .show(ui, |ui| {
                ui.label(
                    RichText::new(format!(
                        "Final Time: {}",
                        format_match_time(session.elapsed_ms)
                    ))
                    .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                    .strong(),
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new(format!("Score: {score} pts"))
                        .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                        .color(p.accent)
                        .strong(),
                );
                ui.add_space(12.0);
                ui.label(format!("Best Streak: {}x", session.best_streak));
                ui.label(format!("Mismatches: {}", session.mismatches));
                if high_score > 0 {
                    ui.label(format!("Session High Score: {high_score} pts"));
                }
            });

        ui.add_space(20.0);
        ui.horizontal(|ui| {
            ui.add_space(ui.available_width() * 0.3);
            if ui
                .button("Play Again")
                .on_hover_text("Starts the game again.")
                .clicked()
            {
                out.push(Msg::Match(MatchMsg::Restart));
            }
            if ui
                .button("Done")
                .on_hover_text("Closes this when you have finished.")
                .clicked()
            {
                out.push(Msg::Match(MatchMsg::Leave));
            }
        });
    });
}

/// Draws the full match mode screen: HUD header, 3x4 grid, and victory modal.
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

    ui.ctx().request_repaint();

    draw_hud(session, ui, out, &pal);
    ui.add_space(16.0);

    if session.completed {
        draw_victory_screen(session, match_state.high_score, ui, out, &pal);
    } else {
        draw_tile_grid(session, ui, out, &pal);
    }
}
