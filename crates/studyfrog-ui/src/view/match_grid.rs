//! Match mode in the redesign style (EPIC-073, EPIC-075): the gradient headline, scope/filter controls,
//! HUD row, and responsive centered tile grid backed by real vault flashcards and questions.
//!
//! Complies with Invariant P1: zero inherent impl blocks; all logic in free functions.

use eframe::egui::{self, Color32, CornerRadius, RichText, Role, Sense, Stroke, Vec2, WidgetInfo};
use studyfrog_rehearsal::match_mode::{MatchModeSession, MatchTile, TileKind, TileStatus};

use crate::match_ui::MatchMsg;
use crate::msg::Msg;
use crate::state::State;
use crate::theme::style::{CARD_RADIUS, Style, style_of};
use crate::theme::{Palette, palette_of};
use crate::view::{gradient_panel, match_controls, match_victory};

const HEADER_HEIGHT: f32 = 64.0;
const TILE_GAP: f32 = 10.0;
const TILE_HEIGHT: f32 = 96.0;

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

fn truncate_tile_text(text: &str, max_chars: usize) -> (String, bool) {
    let count = text.chars().count();
    if count <= max_chars {
        return (text.to_owned(), false);
    }
    let keep = max_chars.saturating_sub(3);
    let mut clipped: String = text.chars().take(keep).collect();
    if let Some(pos) = clipped.rfind(' ')
        && pos > keep / 2
    {
        clipped.truncate(pos);
    }
    clipped.push_str("...");
    (clipped, true)
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
    resp.widget_info(|| WidgetInfo::labeled(Role::Button, true, &tile.text));

    let fill = if resp.hovered() && tile.status == TileStatus::Idle {
        p.surface_hover
    } else {
        bg
    };

    let radius = CornerRadius::same(CARD_RADIUS);
    ui.painter().rect_filled(rect, radius, fill);
    ui.painter()
        .rect_stroke(rect, radius, stroke, egui::StrokeKind::Inside);

    let badge = match tile.kind {
        TileKind::Prompt => "Prompt",
        TileKind::Answer => "Answer",
    };
    let badge_font = crate::typo::font(ui, crate::typo::Tier::Caption);
    let badge_pos = egui::pos2(rect.left() + 10.0, rect.top() + 6.0);
    ui.painter().text(
        badge_pos,
        egui::Align2::LEFT_TOP,
        badge,
        badge_font,
        p.text_weak,
    );

    let inner = rect.shrink2(Vec2::new(10.0, 18.0));
    let max_chars = ((inner.width() / 7.5).max(16.0) * 3.0) as usize;
    let (shown, truncated) = truncate_tile_text(&tile.text, max_chars);

    let body_size = crate::typo::size(ui, crate::typo::Tier::Body);
    let galley = ui.painter().layout(
        shown,
        egui::FontId::proportional(body_size),
        text_color,
        inner.width().max(24.0),
    );
    let clipped_painter = ui.painter().with_clip_rect(inner);
    let y = (inner.center().y - galley.size().y * 0.5).max(inner.top());
    let x = inner.center().x - galley.size().x * 0.5;
    clipped_painter.galley(egui::pos2(x, y), galley, text_color);

    let clicked = if truncated {
        resp.on_hover_text(&tile.text).clicked()
    } else {
        resp.clicked()
    };
    if clicked {
        out.push(Msg::Match(MatchMsg::SelectTile(tile.id)));
    }
}

fn grid_columns(tile_count: usize, avail_width: f32) -> usize {
    if avail_width < 520.0 {
        return 2;
    }
    match tile_count {
        0..=8 => 4.min(tile_count.max(1)),
        9..=12 => {
            if avail_width >= 760.0 {
                4
            } else {
                3
            }
        }
        _ => 4,
    }
}

fn draw_tile_grid(
    session: &MatchModeSession,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
    p: &Palette,
    s: &Style,
) {
    let avail = ui.available_width();
    let cols = grid_columns(session.tiles.len(), avail);
    let gaps = TILE_GAP * cols.saturating_sub(1) as f32;
    let col_width = ((avail - gaps) / cols as f32).clamp(120.0, 280.0);
    let grid_width = col_width * cols as f32 + gaps;
    let left_pad = ((avail - grid_width) * 0.5).max(0.0);
    let tile_size = Vec2::new(col_width, TILE_HEIGHT);
    let rows = session.tiles.len().div_ceil(cols);

    for row in 0..rows {
        ui.horizontal(|ui| {
            if left_pad > 0.0 {
                ui.add_space(left_pad);
            }
            ui.spacing_mut().item_spacing.x = TILE_GAP;
            for col in 0..cols {
                let idx = row * cols + col;
                if let Some(tile) = session.tiles.get(idx) {
                    draw_tile(tile, tile_size, ui, out, p, s);
                }
            }
        });
        ui.add_space(TILE_GAP);
    }
}

/// Draws the full match mode screen: headline, scope/filter controls, HUD, and tile grid or empty/victory state.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(match_state) = state.match_state.as_ref() else {
        out.push(Msg::Match(MatchMsg::StartFromVault));
        return;
    };

    let pal = palette_of(ui);
    let s = style_of(&pal);

    headline(ui);
    ui.add_space(8.0);
    match_controls::draw_controls(state, match_state, ui, out, &pal);
    ui.add_space(8.0);

    if match_state.loading || match_state.error.is_some() || match_state.pairs_source.len() < 2 {
        match_controls::draw_empty_state(match_state, ui, out, &pal);
        return;
    }

    let session = &match_state.session;
    if !session.completed {
        ui.ctx().request_repaint();
    }
    match_controls::draw_hud(match_state, ui, out, &pal);
    ui.add_space(12.0);

    if session.completed {
        match_victory::draw(session, match_state.high_score, ui, out, &pal);
    } else {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                draw_tile_grid(session, ui, out, &pal, &s);
            });
    }
}
