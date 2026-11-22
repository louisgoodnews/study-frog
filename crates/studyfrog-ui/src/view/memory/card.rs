//! Painting one memory card: the back (a dot lattice), the face (wrapped text), the match glow, the
//! mismatch outline and shake, and the keyboard focus ring. The rules are in the engine; this only
//! draws what it is given.

use eframe::egui::{self, Color32, Pos2, Rect, Stroke, StrokeKind, Vec2};
use studyfrog_rehearsal::memory_deck::{FlipState, MemoryTile};

use crate::theme::palette_of;

/// Corner radius of a card, in points.
pub(super) const RADIUS: u8 = 10;

/// What a screen reader is told about a card.
pub(super) fn card_label(i: usize, tile: &MemoryTile) -> String {
    match tile.state {
        FlipState::FaceDown => format!("Card {}, face down", i + 1),
        FlipState::Matched => format!("Card {}, matched: {}", i + 1, tile.text),
        FlipState::FaceUp | FlipState::MismatchWait => format!("Card {}: {}", i + 1, tile.text),
    }
}

/// A mismatched card jiggles left and right while it is shown. With reduced motion (`motion` off)
/// it stays still and keeps only its danger outline.
pub(super) fn shaken(base: Rect, tile: &MemoryTile, elapsed_ms: u64, motion: bool) -> Rect {
    if !motion || tile.state != FlipState::MismatchWait {
        return base;
    }
    let dx = if (elapsed_ms / 70).is_multiple_of(2) {
        -4.0
    } else {
        4.0
    };
    base.translate(Vec2::new(dx, 0.0))
}

/// Paint one card, showing its face when `face` is set (the card has turned past the middle of
/// its flip) and its back otherwise. A face-up card is styled by its state.
pub(super) fn paint(ui: &mut egui::Ui, rect: Rect, tile: &MemoryTile, face: bool, hovered: bool) {
    let pal = palette_of(ui);
    let painter = ui.painter().clone();
    if !face {
        painter.rect_filled(rect, RADIUS, pal.surface);
        dots(&painter, rect, pal.accent.gamma_multiply(0.45));
        let (width, color) = if hovered {
            (2.0, pal.accent)
        } else {
            (1.0, pal.border)
        };
        painter.rect_stroke(rect, RADIUS, Stroke::new(width, color), StrokeKind::Inside);
        return;
    }
    match tile.state {
        FlipState::Matched => {
            painter.rect_stroke(
                rect.expand(3.0),
                RADIUS + 3,
                Stroke::new(3.0, pal.success),
                StrokeKind::Outside,
            );
            painter.rect_filled(rect, RADIUS, pal.success.gamma_multiply(0.18));
        }
        FlipState::MismatchWait | FlipState::FaceUp | FlipState::FaceDown => {
            let edge = if tile.state == FlipState::MismatchWait {
                pal.danger
            } else {
                pal.accent
            };
            painter.rect_filled(rect, RADIUS, pal.panel);
            painter.rect_stroke(rect, RADIUS, Stroke::new(2.0, edge), StrokeKind::Inside);
        }
    }
    text(ui, rect, &tile.text, pal.text);
}

/// A ring around the card that has keyboard focus.
pub(super) fn focus_ring(ui: &egui::Ui, rect: Rect) {
    let pal = palette_of(ui);
    ui.painter().rect_stroke(
        rect.expand(4.0),
        RADIUS + 4,
        Stroke::new(2.0, pal.text),
        StrokeKind::Outside,
    );
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
