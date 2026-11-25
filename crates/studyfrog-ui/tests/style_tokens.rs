#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! STORY-0213: the redesign tokens (`theme::style`) meet the APCA targets of `appearance_apca` in all 2 x 2 x 6
//! palettes: text on the gradient, on a badge and on the rail, and the card outline against the card.

use eframe::egui::Color32;
use studyfrog_core::appearance::ACCENT_LIST;
use studyfrog_ui::color_science::apca_abs;
use studyfrog_ui::theme::style::{CARD_RADIUS, GUTTER, style_of};
use studyfrog_ui::theme::{Look, Palette, blend, palette_with};

/// The targets (standard, high contrast), as in `appearance_apca`.
const ON_ACCENT: (f32, f32) = (60.0, 75.0);
const TEXT_WEAK: (f32, f32) = (58.0, 75.0);
const DIVIDER: (f32, f32) = (15.0, 50.0);

fn all() -> Vec<(String, bool, Palette)> {
    let mut out = Vec::new();
    for dark in [true, false] {
        for high_contrast in [false, true] {
            for accent in ACCENT_LIST {
                let p = palette_with(
                    dark,
                    Look {
                        accent,
                        high_contrast,
                    },
                );
                out.push((
                    format!("dark {dark}, high {high_contrast}, {accent:?}"),
                    high_contrast,
                    p,
                ));
            }
        }
    }
    assert_eq!(out.len(), 24);
    out
}

fn at_least(text: Color32, bg: Color32, goal: f32, who: &str, what: &str) {
    let lc = apca_abs(text, bg);
    assert!(lc >= goal, "{who}: {what} is {lc:.1}, needs {goal}");
}

#[test]
fn text_on_the_gradient_meets_the_on_accent_target_at_both_ends() {
    for (who, high, p) in all() {
        let s = style_of(&p);
        let goal = if high { ON_ACCENT.1 } else { ON_ACCENT.0 };
        for end in s.gradient {
            at_least(
                s.gradient_text,
                end,
                goal,
                &who,
                "gradient text on a gradient end",
            );
        }
    }
}

#[test]
fn text_on_a_badge_meets_the_weak_text_target_on_the_blended_fill() {
    for (who, high, p) in all() {
        let s = style_of(&p);
        let goal = if high { TEXT_WEAK.1 } else { TEXT_WEAK.0 };
        at_least(s.badge_text, s.badge, goal, &who, "badge text");
    }
}

#[test]
fn text_on_the_active_rail_item_meets_the_weak_text_target() {
    for (who, high, p) in all() {
        let s = style_of(&p);
        let goal = if high { TEXT_WEAK.1 } else { TEXT_WEAK.0 };
        at_least(p.text, s.rail_active, goal, &who, "rail item text");
    }
}

#[test]
fn the_card_outline_is_seen_against_the_card() {
    for (who, high, p) in all() {
        let s = style_of(&p);
        let goal = if high { DIVIDER.1 } else { DIVIDER.0 };
        at_least(s.card_edge, s.card, goal, &who, "card outline");
    }
}

#[test]
fn the_ring_progress_is_seen_against_its_track() {
    for (who, high, p) in all() {
        let s = style_of(&p);
        let goal = if high { DIVIDER.1 } else { DIVIDER.0 };
        at_least(s.ring_fill, s.ring_track, goal, &who, "ring progress");
    }
}

#[test]
fn every_gradient_has_a_flat_fallback_and_the_cards_keep_the_gutter() {
    for (who, _, p) in all() {
        let s = style_of(&p);
        assert_eq!(s.gradient_fallback, s.gradient[0], "{who}");
    }
    assert_eq!(GUTTER, 16.0);
    assert_eq!(CARD_RADIUS, 12);
    assert_eq!(blend(Color32::WHITE, 255, Color32::BLACK), Color32::WHITE);
    assert_eq!(blend(Color32::WHITE, 0, Color32::BLACK), Color32::BLACK);
}
