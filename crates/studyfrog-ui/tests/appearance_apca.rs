#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! STORY-0113: every text/background pair the theme uses meets its APCA target (the table in
//! `docs/specs/ui-colour-and-type.md`) in all 2 x 2 x 6 themes, contrasts and accents; accents and status colours
//! share one OKLCH lightness and chroma; surfaces climb in equal lightness steps.

use eframe::egui::Color32;
use studyfrog_core::appearance::{ACCENT_LIST, Accent};
use studyfrog_core::item_color::PALETTE;
use studyfrog_md::hue::HUES;
use studyfrog_ui::color_science::{apca_abs, max_chroma, rgb_to_oklch};
use studyfrog_ui::item_look::card_colors;
use studyfrog_ui::theme::{Look, Palette, palette_with};
use studyfrog_ui::theme_hue::hue_color;
use studyfrog_ui::theme_tokens::{
    ALT_HUE, DANGER_HUE, SUCCESS_HUE, WARNING_HUE, accent_hue, tokens,
};

/// The targets: (standard, high contrast).
const TEXT: (f32, f32) = (75.0, 90.0);
const TEXT_WEAK: (f32, f32) = (58.0, 75.0);
const ON_ACCENT: (f32, f32) = (60.0, 75.0);
const COLOURED_TEXT: (f32, f32) = (45.0, 60.0);
const DIVIDER: (f32, f32) = (15.0, 50.0);
const DIVIDER_MAX: f32 = 28.0;
/// Light text on a dark theme stops short of white on black (halation).
const DARK_TEXT_MAX: f32 = 100.0;

fn all() -> Vec<(String, bool, bool, Palette)> {
    with_accent()
        .into_iter()
        .map(|(w, d, h, _, p)| (w, d, h, p))
        .collect()
}

fn with_accent() -> Vec<(String, bool, bool, Accent, Palette)> {
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
                let who = format!("dark {dark}, high contrast {high_contrast}, {accent:?}");
                out.push((who, dark, high_contrast, accent, p));
            }
        }
    }
    assert_eq!(out.len(), 24);
    out
}

fn at_least(text: Color32, bgs: &[Color32], goal: f32, what: &str) {
    for bg in bgs {
        let lc = apca_abs(text, *bg);
        assert!(
            lc >= goal,
            "{what}: Lc {lc:.1} < {goal} for {text:?} on {bg:?}"
        );
    }
}

#[test]
fn text_and_secondary_text_meet_their_apca_targets() {
    for (who, dark, hc, p) in all() {
        let pick = |t: (f32, f32)| if hc { t.1 } else { t.0 };
        let grounds = [p.background, p.panel, p.surface];
        at_least(
            p.text,
            &[p.background, p.panel, p.surface, p.surface_hover],
            pick(TEXT),
            &format!("text, {who}"),
        );
        at_least(
            p.text_weak,
            &grounds,
            pick(TEXT_WEAK),
            &format!("weak text, {who}"),
        );
        assert!(
            apca_abs(p.text_weak, p.surface) < apca_abs(p.text, p.surface),
            "hierarchy, {who}"
        );
        if dark && !hc {
            assert!(
                apca_abs(p.text, p.background) <= DARK_TEXT_MAX,
                "halation, {who}"
            );
        }
    }
}

#[test]
fn accents_status_colours_and_text_on_the_accent_meet_their_targets() {
    for (who, _, hc, p) in all() {
        let pick = |t: (f32, f32)| if hc { t.1 } else { t.0 };
        at_least(
            p.on_accent,
            &[p.accent],
            pick(ON_ACCENT),
            &format!("on accent, {who}"),
        );
        for c in [p.accent, p.success, p.warning, p.danger, p.alt] {
            at_least(
                c,
                &[p.background, p.panel, p.surface],
                pick(COLOURED_TEXT),
                &format!("coloured text, {who}"),
            );
        }
    }
}

#[test]
fn dividers_are_quiet_but_visible() {
    for (who, _, hc, p) in all() {
        for bg in [p.panel, p.surface] {
            let lc = apca_abs(p.border, bg);
            if hc {
                assert!(lc >= DIVIDER.1, "border Lc {lc:.1}, {who}");
            } else {
                assert!(
                    (DIVIDER.0..=DIVIDER_MAX).contains(&lc),
                    "border Lc {lc:.1}, {who}"
                );
            }
        }
    }
}

#[test]
fn accents_and_status_colours_share_one_lightness_and_chroma() {
    for (who, dark, hc, accent, p) in with_accent() {
        let t = tokens(dark, hc);
        let pairs = [
            (p.accent, accent_hue(accent)),
            (p.success, SUCCESS_HUE),
            (p.warning, WARNING_HUE),
            (p.danger, DANGER_HUE),
            (p.alt, ALT_HUE),
        ];
        for (c, hue) in pairs {
            let o = rgb_to_oklch(c);
            assert!(
                (o.l - t.accent_l).abs() < 0.006,
                "L {} of {c:?}, {who}",
                o.l
            );
            let want = t.accent_c.min(max_chroma(t.accent_l, hue));
            assert!(
                (o.c - want).abs() < 0.012,
                "C {} not {want} for {c:?}, {who}",
                o.c
            );
        }
    }
}

#[test]
fn surfaces_climb_in_equal_lightness_steps() {
    for (who, _, _, p) in all() {
        let l = |c| rgb_to_oklch(c).l;
        let (a, b, c) = (l(p.background), l(p.panel), l(p.surface));
        let step = b - a;
        assert!(step > 0.015, "background to panel {step}, {who}");
        assert!(
            ((c - b) - step).abs() < 0.004,
            "panel to surface {} vs {step}, {who}",
            c - b
        );
        assert!(
            (l(p.surface_hover) - c).abs() >= step - 0.004,
            "hover, {who}"
        );
    }
}

#[test]
fn note_colours_and_item_tints_stay_readable() {
    for (who, dark, hc, p) in all() {
        for hue in HUES {
            let c = hue_color(hue, dark, hc, &p);
            at_least(
                c,
                &[p.panel, p.surface],
                if hc { 90.0 } else { 60.0 },
                &format!("{hue:?}, {who}"),
            );
        }
        for (word, _, _) in PALETTE {
            let (fill, _) = card_colors(&p, dark, Some(word));
            at_least(
                p.text,
                &[fill],
                if hc { 90.0 } else { 75.0 },
                &format!("text on {word}, {who}"),
            );
            at_least(
                p.text_weak,
                &[fill],
                58.0_f32.min(apca_abs(p.text_weak, p.surface)),
                &format!("weak on {word}, {who}"),
            );
        }
    }
}
