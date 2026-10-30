#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The board through update and the real executor, against a real vault.

mod common;

use common::flow::{board, opened, send, titles};
use common::run_to_idle;
use studyfrog_core::uid::Uid;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::worker_new;
use studyfrog_ui::kanban::BoardMsg;
use studyfrog_ui::model::Model;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::{NoticeKind, Screen};

#[test]
fn cards_move_across_the_board_and_wip_limits_hold_until_forced() {
    let (_vault, mut model, mut worker) = opened("board");
    send(&mut model, &mut worker, Msg::Navigate(Screen::Board));
    let cols = model.state.kanban.board.as_ref().unwrap().columns.len();
    assert_eq!(cols, 4);

    for t in ["a", "b", "c", "d"] {
        board(&mut model, &mut worker, BoardMsg::NewTitle(t.into()));
        board(&mut model, &mut worker, BoardMsg::Add);
    }
    assert_eq!(titles(&model, 0), ["a", "b", "c", "d"]);
    let ids: Vec<Uid> = model.state.kanban.board.as_ref().unwrap().columns[0]
        .cards
        .iter()
        .map(|c| c.id)
        .collect();
    for id in &ids[..3] {
        board(&mut model, &mut worker, BoardMsg::Push(*id));
    }
    assert_eq!(
        titles(&model, 1),
        ["c", "b", "a"],
        "pushed cards land on top; Doing holds three"
    );

    board(&mut model, &mut worker, BoardMsg::Push(ids[3]));
    let n = model.state.notice.clone().unwrap();
    assert_eq!(n.kind, NoticeKind::Error);
    assert_eq!(titles(&model, 0), ["d"], "a refused push changes nothing");

    board(&mut model, &mut worker, BoardMsg::Force(true));
    board(&mut model, &mut worker, BoardMsg::Push(ids[3]));
    assert_eq!(titles(&model, 1).len(), 4, "forcing is allowed on request");

    let done = model.state.kanban.board.as_ref().unwrap().columns[3].id;
    board(
        &mut model,
        &mut worker,
        BoardMsg::Move {
            card: ids[0],
            to: done,
            index: 0,
        },
    );
    assert_eq!(titles(&model, 3), ["a"]);
    board(&mut model, &mut worker, BoardMsg::Shift(ids[3], 1));
    assert_eq!(titles(&model, 1), ["c", "d", "b"]);
}

#[test]
fn a_card_is_edited_linked_and_removed_and_the_board_survives_a_restart() {
    let (_vault, mut model, mut worker) = opened("board-edit");
    send(&mut model, &mut worker, Msg::Navigate(Screen::Board));
    board(
        &mut model,
        &mut worker,
        BoardMsg::NewTitle("Read ch. 3".into()),
    );
    board(&mut model, &mut worker, BoardMsg::Add);
    let id = model.state.kanban.board.as_ref().unwrap().columns[0].cards[0].id;

    board(&mut model, &mut worker, BoardMsg::Open(id));
    board(&mut model, &mut worker, BoardMsg::EditPoints("lots".into()));
    board(&mut model, &mut worker, BoardMsg::SaveCard);
    assert!(model.state.notice.is_some(), "a bad estimate is reported");
    assert!(
        model.state.kanban.edit.is_some(),
        "and the editor stays open"
    );

    board(&mut model, &mut worker, BoardMsg::EditPoints("3".into()));
    board(
        &mut model,
        &mut worker,
        BoardMsg::EditTitle("Read chapter 3".into()),
    );
    board(&mut model, &mut worker, BoardMsg::EditStack(Some(Uid(1))));
    board(&mut model, &mut worker, BoardMsg::SaveCard);
    assert!(model.state.kanban.edit.is_none());

    let path = model.state.vault.as_ref().unwrap().root.clone();
    let mut again = worker_new(None);
    let mut fresh = Model::default();
    run_to_idle(&mut fresh, &mut again, vec![Effect::OpenVault(path)]);
    send(&mut fresh, &mut again, Msg::Navigate(Screen::Board));
    let card = &fresh.state.kanban.board.as_ref().unwrap().columns[0].cards[0];
    assert_eq!(
        (card.title.as_str(), card.points, card.stack),
        ("Read chapter 3", Some(3), Some(Uid(1)))
    );

    board(&mut model, &mut worker, BoardMsg::Remove(id));
    assert!(titles(&model, 0).is_empty());
}
