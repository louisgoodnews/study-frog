//! Tests for Match Mode UI integration, message handling, and grid rendering.

use studyfrog_rehearsal::match_mode::{TileKind, TileStatus, compute_match_score};
use studyfrog_ui::match_ui::{MatchMsg, default_match_pairs, new_match_ui, update};
use studyfrog_ui::state::{Screen, State};
use studyfrog_ui::view::match_grid;

fn test_state() -> State {
    State {
        screen: Screen::Match,
        now: 1.5,
        wall_ms: 1_700_000_000,
        ..Default::default()
    }
}

#[test]
fn test_match_ui_initialization_and_start() {
    let state = test_state();
    let pairs = vec![
        ("Term A".into(), "Def A".into()),
        ("Term B".into(), "Def B".into()),
    ];
    let (new_state, _) = update(state, MatchMsg::Start(pairs.clone()));
    assert_eq!(new_state.screen, Screen::Match);
    let ms = new_state.match_state.expect("match_state initialized");
    assert_eq!(ms.pairs_source, pairs);
    assert_eq!(ms.session.pairs_remaining, 2);
    assert_eq!(ms.session.tiles.len(), 4);
}

#[test]
fn test_match_ui_default_pairs() {
    let pairs = default_match_pairs();
    assert_eq!(pairs.len(), 6);
    let ui_state = new_match_ui(Vec::new(), 0, 1234);
    assert_eq!(ui_state.session.tiles.len(), 12);
    assert_eq!(ui_state.session.pairs_remaining, 6);
}

#[test]
fn test_match_ui_selection_and_streak() {
    let mut state = test_state();
    let (s1, _) = update(state, MatchMsg::Start(default_match_pairs()));
    state = s1;

    let p0 = state
        .match_state
        .as_ref()
        .unwrap()
        .session
        .tiles
        .iter()
        .position(|t| t.pair_id == 0 && t.kind == TileKind::Prompt)
        .unwrap();
    let a0 = state
        .match_state
        .as_ref()
        .unwrap()
        .session
        .tiles
        .iter()
        .position(|t| t.pair_id == 0 && t.kind == TileKind::Answer)
        .unwrap();

    let (s2, _) = update(state, MatchMsg::SelectTile(p0));
    state = s2;
    assert_eq!(
        state.match_state.as_ref().unwrap().session.tiles[p0].status,
        TileStatus::Selected
    );

    let (s3, _) = update(state, MatchMsg::SelectTile(a0));
    state = s3;
    assert_eq!(
        state.match_state.as_ref().unwrap().session.tiles[p0].status,
        TileStatus::Matched
    );
    assert_eq!(
        state.match_state.as_ref().unwrap().session.tiles[a0].status,
        TileStatus::Matched
    );
    assert_eq!(state.match_state.as_ref().unwrap().session.streak, 1);
}

#[test]
fn test_match_ui_restart_preserves_high_score() {
    let mut state = test_state();
    let (s1, _) = update(state, MatchMsg::Start(default_match_pairs()));
    state = s1;

    // Force match completion to generate score
    for pair_id in 0..6 {
        let p = state
            .match_state
            .as_ref()
            .unwrap()
            .session
            .tiles
            .iter()
            .position(|t| t.pair_id == pair_id && t.kind == TileKind::Prompt)
            .unwrap();
        let a = state
            .match_state
            .as_ref()
            .unwrap()
            .session
            .tiles
            .iter()
            .position(|t| t.pair_id == pair_id && t.kind == TileKind::Answer)
            .unwrap();
        let (s_sel, _) = update(state, MatchMsg::SelectTile(p));
        let (s_match, _) = update(s_sel, MatchMsg::SelectTile(a));
        state = s_match;
    }

    let ms = state.match_state.as_ref().unwrap();
    assert!(ms.session.completed);
    let score = compute_match_score(&ms.session);
    assert!(score > 0);
    assert_eq!(ms.high_score, score);

    // Restart should create fresh tiles but retain high score
    let (s_restarted, _) = update(state, MatchMsg::Restart);
    let ms_restarted = s_restarted.match_state.as_ref().unwrap();
    assert!(!ms_restarted.session.completed);
    assert_eq!(ms_restarted.session.pairs_remaining, 6);
    assert_eq!(ms_restarted.high_score, score);
}

#[test]
fn test_match_ui_leave() {
    let state = test_state();
    let (new_state, _) = update(state, MatchMsg::Leave);
    assert_eq!(new_state.screen, Screen::Home);
}

#[test]
fn test_match_grid_view_rendering() {
    let mut state = test_state();
    let (s1, _) = update(state, MatchMsg::Start(default_match_pairs()));
    state = s1;

    let ctx = eframe::egui::Context::default();
    let mut out = Vec::new();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        match_grid::draw(&state, ui, &mut out);
    });
    output.textures_delta.clear();
}
