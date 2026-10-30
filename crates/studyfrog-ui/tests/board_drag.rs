#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Dragging a board card to another column with the pointer.

mod common;

use common::views::{harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_board::edit::add_card;
use studyfrog_board::model::{MoveMode, board_default, card_new};
use studyfrog_core::uid::Uid;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::kanban::BoardMsg;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

fn model() -> Model {
    let mut m = open_model();
    m.state.screen = Screen::Board;
    let ids = [Uid(21), Uid(22), Uid(23), Uid(24)];
    let mut b = board_default(ids);
    for (i, t) in ["Read", "Write"].into_iter().enumerate() {
        b = add_card(
            &b,
            ids[0],
            card_new(Uid(100 + i as u128), t),
            MoveMode::Force,
        )
        .unwrap();
    }
    apply(&mut m, Msg::Board(BoardMsg::Loaded(Ok(b))));
    m
}

#[test]
fn dragging_a_card_onto_another_column_moves_it_there() {
    let mut h = harness(model());
    h.run();
    let from = h.get_by_label("Read").rect().center();
    let target = h.get_by_label_contains("Doing").rect();
    let to = eframe::egui::pos2(target.center().x, target.bottom() + 40.0);
    h.hover_at(from);
    h.run();
    h.drag_at(from);
    h.run();
    for step in 1..=8 {
        let t = step as f32 / 8.0;
        h.hover_at(from + (to - from) * t);
        h.run();
    }
    h.drop_at(to);
    h.run();
    let moved = h.state().1.iter().any(|e| {
        matches!(
            e,
            Effect::BoardCommand(studyfrog_board::ops::BoardOp::Move {
                card: Uid(100),
                to: Uid(22),
                ..
            })
        )
    });
    assert!(
        moved,
        "a Move effect for the first card into Doing: {:?}",
        h.state().1
    );
}
