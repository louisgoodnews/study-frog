//! Tests for match mode state machine and mechanics.

use studyfrog_rehearsal::match_mode::{
    MatchOutcome, TileKind, TileStatus, compute_match_score, format_match_time, match_session_new,
    select_tile, update_match_timer,
};

fn sample_pairs() -> Vec<(String, String)> {
    vec![
        ("Mitochondria".into(), "Powerhouse of the cell".into()),
        ("Ribosome".into(), "Protein synthesis".into()),
        ("Nucleus".into(), "Genetic material".into()),
        ("Chloroplast".into(), "Photosynthesis".into()),
        ("Vacuole".into(), "Storage organelle".into()),
        ("Golgi".into(), "Packaging and secretion".into()),
    ]
}

#[test]
fn test_match_session_initialization() {
    let pairs = sample_pairs();
    let session = match_session_new(&pairs, 1000, 42);
    assert_eq!(session.tiles.len(), 12);
    assert_eq!(session.pairs_remaining, 6);
    assert_eq!(session.start_t_ms, 1000);
    assert_eq!(session.elapsed_ms, 0);
    assert_eq!(session.penalty_ms, 0);
    assert_eq!(session.streak, 0);
    assert_eq!(session.best_streak, 0);
    assert_eq!(session.mismatches, 0);
    assert!(!session.completed);

    let prompts = session
        .tiles
        .iter()
        .filter(|t| t.kind == TileKind::Prompt)
        .count();
    let answers = session
        .tiles
        .iter()
        .filter(|t| t.kind == TileKind::Answer)
        .count();
    assert_eq!(prompts, 6);
    assert_eq!(answers, 6);
    for (i, t) in session.tiles.iter().enumerate() {
        assert_eq!(t.id, i);
        assert_eq!(t.status, TileStatus::Idle);
    }
}

#[test]
fn test_match_session_empty_initialization() {
    let session = match_session_new(&[], 500, 1);
    assert!(session.tiles.is_empty());
    assert_eq!(session.pairs_remaining, 0);
    assert!(session.completed);
    assert_eq!(compute_match_score(&session), 0);
}

#[test]
fn test_tile_selection_and_deselection() {
    let pairs = sample_pairs();
    let mut session = match_session_new(&pairs, 0, 42);

    let res = select_tile(&mut session, 0, 100);
    assert_eq!(res, MatchOutcome::Selected(0));
    assert_eq!(session.selected_first, Some(0));
    assert_eq!(session.tiles[0].status, TileStatus::Selected);

    let res2 = select_tile(&mut session, 0, 200);
    assert_eq!(res2, MatchOutcome::Deselected);
    assert_eq!(session.selected_first, None);
    assert_eq!(session.tiles[0].status, TileStatus::Idle);
    assert_eq!(session.penalty_ms, 0);
}

#[test]
fn test_successful_pair_match() {
    let pairs = sample_pairs();
    let mut session = match_session_new(&pairs, 0, 42);

    let prompt_idx = session
        .tiles
        .iter()
        .position(|t| t.pair_id == 0 && t.kind == TileKind::Prompt)
        .unwrap();
    let answer_idx = session
        .tiles
        .iter()
        .position(|t| t.pair_id == 0 && t.kind == TileKind::Answer)
        .unwrap();

    let res1 = select_tile(&mut session, prompt_idx, 300);
    assert_eq!(res1, MatchOutcome::Selected(prompt_idx));

    let res2 = select_tile(&mut session, answer_idx, 600);
    assert_eq!(
        res2,
        MatchOutcome::Matched {
            pair_id: 0,
            streak: 1
        }
    );
    assert_eq!(session.selected_first, None);
    assert_eq!(session.tiles[prompt_idx].status, TileStatus::Matched);
    assert_eq!(session.tiles[answer_idx].status, TileStatus::Matched);
    assert_eq!(session.pairs_remaining, 5);
    assert_eq!(session.streak, 1);
    assert_eq!(session.best_streak, 1);
}

#[test]
fn test_mismatch_penalty_and_reset() {
    let pairs = sample_pairs();
    let mut session = match_session_new(&pairs, 1000, 42);

    let tile_a = session.tiles.iter().position(|t| t.pair_id == 0).unwrap();
    let tile_b = session.tiles.iter().position(|t| t.pair_id == 1).unwrap();

    select_tile(&mut session, tile_a, 1500);
    let outcome = select_tile(&mut session, tile_b, 2000);
    assert_eq!(outcome, MatchOutcome::Mismatch { penalty_ms: 1000 });
    assert_eq!(session.mismatches, 1);
    assert_eq!(session.penalty_ms, 1000);
    assert_eq!(session.streak, 0);
    assert_eq!(
        session.tiles[tile_a].status,
        TileStatus::Mismatch { until_ms: 3000 }
    );
    assert_eq!(
        session.tiles[tile_b].status,
        TileStatus::Mismatch { until_ms: 3000 }
    );

    update_match_timer(&mut session, 2500);
    assert_eq!(
        session.tiles[tile_a].status,
        TileStatus::Mismatch { until_ms: 3000 }
    );

    update_match_timer(&mut session, 3001);
    assert_eq!(session.tiles[tile_a].status, TileStatus::Idle);
    assert_eq!(session.tiles[tile_b].status, TileStatus::Idle);
}

#[test]
fn test_mismatch_cleared_by_early_selection() {
    let pairs = sample_pairs();
    let mut session = match_session_new(&pairs, 0, 42);

    let tile_a = session.tiles.iter().position(|t| t.pair_id == 0).unwrap();
    let tile_b = session.tiles.iter().position(|t| t.pair_id == 1).unwrap();
    let tile_c = session.tiles.iter().position(|t| t.pair_id == 2).unwrap();

    select_tile(&mut session, tile_a, 100);
    select_tile(&mut session, tile_b, 200);

    let res = select_tile(&mut session, tile_c, 300);
    assert_eq!(res, MatchOutcome::Selected(tile_c));
    assert_eq!(session.tiles[tile_a].status, TileStatus::Idle);
    assert_eq!(session.tiles[tile_b].status, TileStatus::Idle);
    assert_eq!(session.tiles[tile_c].status, TileStatus::Selected);
}

#[test]
fn test_full_session_completion() {
    let pairs = sample_pairs();
    let mut session = match_session_new(&pairs, 0, 42);

    for pair_id in 0..6 {
        let p = session
            .tiles
            .iter()
            .position(|t| t.pair_id == pair_id && t.kind == TileKind::Prompt)
            .unwrap();
        let a = session
            .tiles
            .iter()
            .position(|t| t.pair_id == pair_id && t.kind == TileKind::Answer)
            .unwrap();
        select_tile(&mut session, p, (pair_id as u32) * 1000);
        let outcome = select_tile(&mut session, a, (pair_id as u32) * 1000 + 500);
        if pair_id == 5 {
            assert!(matches!(outcome, MatchOutcome::Completed { .. }));
        }
    }

    assert!(session.completed);
    assert_eq!(session.pairs_remaining, 0);
    assert_eq!(session.streak, 6);
    assert_eq!(session.best_streak, 6);

    let ignored = select_tile(&mut session, 0, 10_000);
    assert_eq!(ignored, MatchOutcome::Ignored);

    let score = compute_match_score(&session);
    assert!(score > 6000);
}

#[test]
fn test_format_match_time() {
    assert_eq!(format_match_time(0), "00:00.0");
    assert_eq!(format_match_time(14_350), "00:14.3");
    assert_eq!(format_match_time(65_200), "01:05.2");
}
