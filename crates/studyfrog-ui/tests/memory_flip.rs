#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The memory card's flip (STORY-0174): the width and side shown through a turn, and reduced motion.

use studyfrog_ui::setup::memory_flip::{
    FLIP_SECS, flip_secs, settled_progress, shows_face, width_fraction,
};

#[test]
fn the_card_is_full_width_at_either_face_and_narrow_mid_turn() {
    assert!((width_fraction(0.0) - 1.0).abs() < 1e-6);
    assert!((width_fraction(1.0) - 1.0).abs() < 1e-6);
    assert!(width_fraction(0.5) < 0.05);
}

#[test]
fn the_face_shows_from_the_middle_of_the_turn() {
    assert!(!shows_face(0.0));
    assert!(!shows_face(0.49));
    assert!(shows_face(0.5));
    assert!(shows_face(1.0));
}

#[test]
fn reduced_motion_makes_the_flip_instant() {
    assert_eq!(flip_secs(0.0), 0.0);
    assert_eq!(flip_secs(1.0 / 12.0), FLIP_SECS);
}

#[test]
fn settled_progress_is_the_face_for_a_face_up_card() {
    assert_eq!(settled_progress(true), 1.0);
    assert_eq!(settled_progress(false), 0.0);
}

#[test]
fn out_of_range_progress_is_clamped() {
    assert!(shows_face(2.0));
    assert!(!shows_face(-1.0));
    assert!((width_fraction(3.0) - 1.0).abs() < 1e-6);
}
