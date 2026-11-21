//! The memory game's board: the card grid, the HUD line and the end-of-game line. The rules are in
//! the engine; this only draws the state it is given and reports presses.

use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2, WidgetInfo};
use studyfrog_rehearsal::memory::{MemoryOutcome, MemorySession};
use studyfrog_rehearsal::memory_deck::{FlipState, MemoryTile};

use crate::msg::Msg;
use crate::setup::memory::{MemoryMsg, MemoryUi};
use crate::setup::memory_layout::{GAP, grid_for, grid_height, grid_width};
use crate::theme::palette_of;
use crate::view::kit;

/// Corner radius of a card, in points.
const RADIUS: u8 = 10;
/// Upper bound on the board's height, so cards stay a sensible size on tall windows.
const MAX_BOARD_H: f32 = 900.0;

/// Draw the game on screen: the HUD with a close button, the end line and the board, or the loading
/// or error line while there is no game yet.
pub fn draw(ui: &mut egui::Ui, memory: &MemoryUi, out: &mut Vec<Msg>) {
    ui.horizontal(|ui| {
        match &memory.session {
            Some(session) => hud(ui, session),
            None if memory.loading => {
                ui.label(crate::typo::text(
                    ui,
                    "Reading the cards…",
                    crate::typo::Tier::Body,
                ));
            }
            None => {}
        }
        if kit::quiet_button(ui, "Close game")
            .on_hover_text("Closes the memory game and returns to the setup.")
            .clicked()
        {
            out.push(Msg::Memory(MemoryMsg::Close));
        }
    });
    if let Some(error) = &memory.error {
        let pal = palette_of(ui);
        ui.colored_label(pal.danger, error);
    }
    if let Some(session) = &memory.session {
        end_line(ui, session, out);
        board(ui, session, out);
        if session.outcome.is_none() {
            out.push(Msg::Memory(MemoryMsg::Clock));
            ui.ctx().request_repaint();
        }
    }
}

/// The time, moves, pairs still to find and the current streak.
fn hud(ui: &mut egui::Ui, s: &MemorySession) {
    let unmatched = s
        .tiles
        .iter()
        .filter(|t| t.state != FlipState::Matched)
        .count();
    let line = format!(
        "Time {}  ·  Moves {}  ·  Pairs left {}  ·  Streak {}",
        clock_text(s.elapsed_ms),
        s.moves,
        unmatched / 2,
        s.streak
    );
    ui.label(crate::typo::text(ui, line, crate::typo::Tier::Body));
}

/// Say how the game ended, with a button to play again.
fn end_line(ui: &mut egui::Ui, s: &MemorySession, out: &mut Vec<Msg>) {
    let text = match s.outcome {
        None => return,
        Some(MemoryOutcome::Completed { elapsed_ms }) => {
            format!("All pairs found in {}.", clock_text(elapsed_ms))
        }
        Some(MemoryOutcome::TimedOut) => "Time is up.".to_owned(),
    };
    ui.horizontal(|ui| {
        ui.label(crate::typo::text(ui, text, crate::typo::Tier::Body));
        if kit::primary_button(ui, "Play again")
            .on_hover_text("Shuffles the same cards into a new game.")
            .clicked()
        {
            out.push(Msg::Memory(MemoryMsg::Start));
        }
    });
}

/// The grid of cards, centred in the space left below the HUD.
fn board(ui: &mut egui::Ui, s: &MemorySession, out: &mut Vec<Msg>) {
    let avail = ui.available_size();
    let height = avail.y.clamp(0.0, MAX_BOARD_H);
    let Some(grid) = grid_for(s.tiles.len(), avail.x, height) else {
        return;
    };
    let (area, _) = ui.allocate_exact_size(Vec2::new(avail.x, grid_height(&grid)), Sense::hover());
    let origin = Pos2::new(area.center().x - grid_width(&grid) / 2.0, area.top());
    for (i, tile) in s.tiles.iter().enumerate() {
        let (row, col) = (i / grid.cols, i % grid.cols);
        let at = origin
            + Vec2::new(
                col as f32 * (grid.tile_w + GAP),
                row as f32 * (grid.tile_h + GAP),
            );
        let base = Rect::from_min_size(at, Vec2::new(grid.tile_w, grid.tile_h));
        let rect = shaken(base, tile, s.elapsed_ms);
        let resp = ui.interact(rect, ui.scope_id().with(("memory-card", i)), Sense::click());
        let label = card_label(i, tile);
        resp.widget_info(|| WidgetInfo::labeled(egui::Role::Button, true, label.clone()));
        let hovered = resp.hovered() && tile.state == FlipState::FaceDown;
        paint(ui, rect, tile, hovered);
        if resp.clicked() && tile.state == FlipState::FaceDown {
            out.push(Msg::Memory(MemoryMsg::Flip(i)));
        }
    }
}

/// A mismatched card jiggles left and right while it is shown.
fn shaken(base: Rect, tile: &MemoryTile, elapsed_ms: u64) -> Rect {
    if tile.state != FlipState::MismatchWait {
        return base;
    }
    let dx = if (elapsed_ms / 70).is_multiple_of(2) {
        -4.0
    } else {
        4.0
    };
    base.translate(Vec2::new(dx, 0.0))
}

fn card_label(i: usize, tile: &MemoryTile) -> String {
    match tile.state {
        FlipState::FaceDown => format!("Card {}, face down", i + 1),
        FlipState::Matched => format!("Card {}, matched: {}", i + 1, tile.text),
        FlipState::FaceUp | FlipState::MismatchWait => format!("Card {}: {}", i + 1, tile.text),
    }
}

/// Paint one card. Face-down cards show a dot lattice; face-up text wraps inside the card.
fn paint(ui: &mut egui::Ui, rect: Rect, tile: &MemoryTile, hovered: bool) {
    let pal = palette_of(ui);
    let painter = ui.painter().clone();
    match tile.state {
        FlipState::FaceDown => {
            painter.rect_filled(rect, RADIUS, pal.surface);
            dots(&painter, rect, pal.accent.gamma_multiply(0.45));
            let (width, color) = if hovered {
                (2.0, pal.accent)
            } else {
                (1.0, pal.border)
            };
            painter.rect_stroke(rect, RADIUS, Stroke::new(width, color), StrokeKind::Inside);
        }
        FlipState::FaceUp | FlipState::MismatchWait => {
            painter.rect_filled(rect, RADIUS, pal.panel);
            let edge = if tile.state == FlipState::MismatchWait {
                pal.danger
            } else {
                pal.accent
            };
            painter.rect_stroke(rect, RADIUS, Stroke::new(2.0, edge), StrokeKind::Inside);
            text(ui, rect, &tile.text, pal.text);
        }
        FlipState::Matched => {
            let glow = rect.expand(3.0);
            painter.rect_stroke(
                glow,
                RADIUS + 3,
                Stroke::new(3.0, pal.success),
                StrokeKind::Outside,
            );
            painter.rect_filled(rect, RADIUS, pal.success.gamma_multiply(0.18));
            text(ui, rect, &tile.text, pal.text);
        }
    }
}

/// Dots on a lattice inside the card: a quiet pattern for the back.
fn dots(painter: &egui::Painter, rect: Rect, color: Color32) {
    const STEP: f32 = 14.0;
    let inner = rect.shrink(10.0);
    let mut y = inner.top();
    let mut row = 0;
    while y <= inner.bottom() {
        let offset = if row % 2 == 0 { 0.0 } else { STEP / 2.0 };
        let mut x = inner.left() + offset;
        while x <= inner.right() {
            painter.circle_filled(Pos2::new(x, y), 1.6, color);
            x += STEP;
        }
        y += STEP;
        row += 1;
    }
}

/// Wrapped text, inset from the card's edge and clipped to it.
fn text(ui: &egui::Ui, rect: Rect, s: &str, color: Color32) {
    let font = crate::typo::font(ui, crate::typo::Tier::Body);
    let wrap = (rect.width() - 16.0).max(1.0);
    let galley = ui.painter().layout(s.to_owned(), font, color, wrap);
    ui.painter_at(rect)
        .galley(rect.left_top() + Vec2::splat(8.0), galley, color);
}

/// Minutes and seconds, e.g. `2:05`.
fn clock_text(ms: u64) -> String {
    let secs = ms / 1000;
    format!("{}:{:02}", secs / 60, secs % 60)
}
