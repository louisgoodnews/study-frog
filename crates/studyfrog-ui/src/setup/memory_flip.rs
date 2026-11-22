//! The flip of a memory card: how wide the card is and which side shows at each point of the turn.
//! Pure: no egui. The board feeds in the animated progress (0 = back, 1 = face) and the frame's
//! animation time, so reduced motion (which sets that time to zero) turns cards over at once.

/// Seconds a card takes to turn over.
pub const FLIP_SECS: f32 = 0.2;
/// The narrowest a turning card gets, as a fraction of its width, so it never vanishes.
const MIN_WIDTH: f32 = 0.04;

/// The seconds a flip takes, given the animation time the app is using. Reduced motion sets the
/// animation time to zero, which makes the flip instant.
pub fn flip_secs(animation_time: f32) -> f32 {
    if animation_time > 0.0 { FLIP_SECS } else { 0.0 }
}

/// The card's width as a fraction of its full width at `progress`: full at either face, zero when
/// the card is edge-on at the middle of the turn.
pub fn width_fraction(progress: f32) -> f32 {
    (2.0 * progress.clamp(0.0, 1.0) - 1.0).abs().max(MIN_WIDTH)
}

/// Whether the face shows at `progress`: past the middle of the turn, the card shows its face.
pub fn shows_face(progress: f32) -> bool {
    progress.clamp(0.0, 1.0) >= 0.5
}

/// The progress a card reaches with no animation: the face for a face-up card, the back otherwise.
pub fn settled_progress(face_up: bool) -> f32 {
    if face_up { 1.0 } else { 0.0 }
}
