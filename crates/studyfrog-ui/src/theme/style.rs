//! The redesign's visual tokens (EPIC-070, STORY-0213): cards, the navigation rail, the stat ring, the gradient
//! headline card, avatars and badges. Pure: every colour is derived from a [`Palette`], so no screen names a colour
//! literal and the contrast tests cover every pair that carries text.

use eframe::egui::Color32;

use crate::theme::{Palette, blend};

/// Corner radius of a card, larger than the controls' so cards read as containers.
pub const CARD_RADIUS: u8 = 12;
/// Gutter at the edge of a screen.
pub const GUTTER: f32 = 16.0;
/// Opacity of a tinted fill behind an accent-coloured mark (a badge, the active rail item).
pub const TINT_ALPHA: u8 = 40;

/// The redesign's colours for one palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    /// Card fill (the surface).
    pub card: Color32,
    /// Card outline.
    pub card_edge: Color32,
    /// The navigation rail's fill.
    pub rail: Color32,
    /// The active rail item's fill.
    pub rail_active: Color32,
    /// The stat ring's track.
    pub ring_track: Color32,
    /// The stat ring's progress.
    pub ring_fill: Color32,
    /// The headline card's gradient, from `gradient[0]` to `gradient[1]`.
    pub gradient: [Color32; 2],
    /// The headline card's flat fallback, for a renderer without gradients.
    pub gradient_fallback: Color32,
    /// Text on the headline card.
    pub gradient_text: Color32,
    /// An avatar's ring or placeholder fill.
    pub avatar: Color32,
    /// A badge's fill.
    pub badge: Color32,
    /// Text on a badge.
    pub badge_text: Color32,
}

/// The redesign's colours for `p`, derived from its palette.
pub fn style_of(p: &Palette) -> Style {
    Style {
        card: p.surface,
        card_edge: p.border,
        rail: p.panel,
        rail_active: blend(p.accent, TINT_ALPHA, p.panel),
        ring_track: p.surface_hover,
        ring_fill: p.accent,
        gradient: [p.accent, p.alt],
        gradient_fallback: p.accent,
        gradient_text: p.on_accent,
        avatar: p.surface_hover,
        badge: blend(p.accent, TINT_ALPHA, p.surface),
        badge_text: p.text,
    }
}
