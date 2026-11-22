//! The memory game's board: the card grid, the HUD line and the end-of-game line. The rules are in
//! the engine; this only draws the state it is given and reports presses.

use eframe::egui::{self, Pos2, Rect, Sense, Vec2, WidgetInfo};
use studyfrog_rehearsal::memory::MemorySession;
use studyfrog_rehearsal::memory_deck::FlipState;
use studyfrog_rehearsal::memory_summary::{DangerLevel, danger_level, hud_clock_ms};

use super::card::{card_label, focus_ring, paint, shaken};
use crate::msg::Msg;
use crate::setup::memory::{MemoryMsg, MemoryUi};
use crate::setup::memory_flip::{flip_secs, settled_progress, shows_face, width_fraction};
use crate::setup::memory_layout::{GAP, grid_for, grid_height, grid_width};
use crate::theme::palette_of;
use crate::view::kit;

/// Upper bound on the board's height, so cards stay a sensible size on tall windows.
const MAX_BOARD_H: f32 = 900.0;

/// Draw the game on screen: the HUD with a close button, the end line and the board, or the loading
/// or error line while there is no game yet.
pub fn draw(ui: &mut egui::Ui, memory: &MemoryUi, out: &mut Vec<Msg>) {
    ui.horizontal(|ui| {
        match &memory.session {
            Some(session) => hud(ui, session, memory),
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
        super::end::end_line(ui, session, out);
        board(ui, session, out);
        if session.outcome.is_none() {
            out.push(Msg::Memory(MemoryMsg::Clock));
            ui.ctx().request_repaint();
        }
    }
}

/// The time (coloured by how urgent the countdown is), moves, pairs still to find and the streak.
fn hud(ui: &mut egui::Ui, s: &MemorySession, memory: &MemoryUi) {
    let pal = palette_of(ui);
    let unmatched = s
        .tiles
        .iter()
        .filter(|t| t.state != FlipState::Matched)
        .count();
    let time_color = match danger_level(memory.timer, s.elapsed_ms) {
        DangerLevel::Calm => pal.text,
        DangerLevel::Warning => pal.warning,
        DangerLevel::Critical => pal.danger,
    };
    let body = crate::typo::Tier::Body;
    ui.horizontal(|ui| {
        let shown = hud_clock_ms(memory.timer, s.elapsed_ms);
        let time = format!("Time {}", clock_text(shown));
        ui.label(crate::typo::text(ui, time, body).color(time_color));
        let rest = format!(
            "·  Moves {}  ·  Pairs left {}  ·  Streak {}",
            s.moves,
            unmatched / 2,
            s.streak
        );
        ui.label(crate::typo::text(ui, rest, body));
    });
}

/// The grid of cards, centred in the space left below the HUD. Each card is a button: a mouse click,
/// or Tab to reach it and Enter or Space to press it, and the arrow keys move between cards.
fn board(ui: &mut egui::Ui, s: &MemorySession, out: &mut Vec<Msg>) {
    let avail = ui.available_size();
    let height = avail.y.clamp(0.0, MAX_BOARD_H);
    let Some(grid) = grid_for(s.tiles.len(), avail.x, height) else {
        return;
    };
    let (area, _) = ui.allocate_exact_size(Vec2::new(avail.x, grid_height(&grid)), Sense::hover());
    let origin = Pos2::new(area.center().x - grid_width(&grid) / 2.0, area.top());
    // Animation time is zero under reduced motion: the cards then turn and shake no more.
    let motion = flip_secs(ui.style().animation_time);
    for (i, tile) in s.tiles.iter().enumerate() {
        let (row, col) = (i / grid.cols, i % grid.cols);
        let at = origin
            + Vec2::new(
                col as f32 * (grid.tile_w + GAP),
                row as f32 * (grid.tile_h + GAP),
            );
        let base = Rect::from_min_size(at, Vec2::new(grid.tile_w, grid.tile_h));
        let rect = shaken(base, tile, s.elapsed_ms, motion > 0.0);
        let card = ui.scope_id().with(("memory-card", i));
        let resp = ui.interact(rect, card, Sense::click());
        resp.widget_info(|| WidgetInfo::labeled(egui::Role::Button, true, card_label(i, tile)));
        let progress = turn(ui, card, tile.state != FlipState::FaceDown, motion);
        let turning = Rect::from_center_size(
            rect.center(),
            Vec2::new(rect.width() * width_fraction(progress), rect.height()),
        );
        let hovered = resp.hovered() && tile.state == FlipState::FaceDown;
        paint(ui, turning, tile, shows_face(progress), hovered);
        if resp.has_focus() {
            focus_ring(ui, turning);
        }
        if resp.clicked() && tile.state == FlipState::FaceDown {
            out.push(Msg::Memory(MemoryMsg::Flip(i)));
        }
    }
}

/// The card's progress towards its face (0 = back, 1 = face) this frame. Over `motion` seconds it
/// eases to the state's side; with no motion it is at the side at once.
fn turn(ui: &egui::Ui, card: egui::Id, face_up: bool, motion: f32) -> f32 {
    let target = settled_progress(face_up);
    if motion > 0.0 {
        ui.ctx()
            .animate_value_with_time(card.with("flip"), target, motion)
    } else {
        target
    }
}

/// Minutes and seconds, e.g. `2:05`.
pub(super) fn clock_text(ms: u64) -> String {
    let secs = ms / 1000;
    format!("{}:{:02}", secs / 60, secs % 60)
}
