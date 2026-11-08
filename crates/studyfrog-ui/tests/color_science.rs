#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The Oklab, OKLCH and APCA maths against published reference values, and its properties.

use eframe::egui::Color32;
use proptest::prelude::*;
use studyfrog_ui::color_science::{
    apca_lc, in_gamut, lch, max_chroma, mix_oklab, oklch_to_rgb, rgb_to_oklab, rgb_to_oklch,
};

fn hex(v: u32) -> Color32 {
    Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

fn near(a: f64, b: f64, tol: f64, what: &str) {
    assert!((a - b).abs() <= tol, "{what}: {a} is not {b} (± {tol})");
}

/// Oklab of the sRGB primaries and white, as CSS Color 4 and Ottosson's reference implementation give them.
#[test]
fn oklab_matches_the_reference_values() {
    let cases = [
        (0xffffff, [1.0, 0.0, 0.0]),
        (0x000000, [0.0, 0.0, 0.0]),
        (0xff0000, [0.627_955, 0.224_863, 0.125_846]),
        (0x00ff00, [0.866_440, -0.233_888, 0.179_498]),
        (0x0000ff, [0.452_014, -0.032_457, -0.311_528]),
    ];
    for (c, want) in cases {
        let got = rgb_to_oklab(hex(c));
        for i in 0..3 {
            near(got[i], want[i], 2e-4, &format!("{c:06x}[{i}]"));
        }
    }
}

#[test]
fn oklch_of_red_has_the_published_hue_and_chroma() {
    let o = rgb_to_oklch(hex(0xff0000));
    near(f64::from(o.l), 0.627_955, 2e-4, "L");
    near(f64::from(o.c), 0.257_683, 2e-4, "C");
    near(f64::from(o.h), 29.234, 0.05, "h");
}

/// The examples of the APCA-W3 reference implementation (0.0.98G-4g), both polarities.
#[test]
fn apca_matches_the_reference_implementation() {
    let cases = [
        (0x888888, 0xffffff, 63.056),
        (0xffffff, 0x888888, -68.541),
        (0x000000, 0xaaaaaa, 58.146),
        (0xaaaaaa, 0x000000, -56.241),
        (0x112233, 0xddeeff, 91.668),
        (0xddeeff, 0x112233, -93.068),
        (0x000000, 0xffffff, 106.04),
        (0xffffff, 0x000000, -107.88),
    ];
    for (t, b, want) in cases {
        let got = f64::from(apca_lc(hex(t), hex(b)));
        near(got, want, 0.05, &format!("{t:06x} on {b:06x}"));
    }
}

#[test]
fn apca_clips_contrast_too_low_to_read_to_zero() {
    assert_eq!(apca_lc(hex(0x777777), hex(0x777777)), 0.0);
    assert_eq!(apca_lc(hex(0x101010), hex(0x141414)), 0.0);
}

#[test]
fn gamut_mapping_keeps_lightness_and_hue_and_lowers_chroma() {
    // Teal at a light-theme lightness cannot have much chroma in sRGB.
    let wanted = lch(0.48, 0.20, 195.0);
    assert!(!in_gamut(wanted));
    let c = max_chroma(0.48, 195.0);
    assert!(c > 0.05 && c < 0.20, "{c}");
    let got = rgb_to_oklch(oklch_to_rgb(wanted));
    near(f64::from(got.l), 0.48, 0.006, "L");
    near(f64::from(got.h), 195.0, 2.0, "h");
    near(f64::from(got.c), f64::from(c), 0.01, "C");
}

#[test]
fn mixing_in_oklab_ends_at_both_colours() {
    let (a, b) = (hex(0x202428), hex(0x40c0a0));
    assert_eq!(mix_oklab(a, b, 0.0), a);
    assert_eq!(mix_oklab(a, b, 1.0), b);
}

proptest! {
    /// sRGB to OKLCH and back gives the same 8-bit colour.
    #[test]
    fn rgb_round_trips_through_oklch(r in 0u8..=255, g in 0u8..=255, b in 0u8..=255) {
        let c = Color32::from_rgb(r, g, b);
        prop_assert_eq!(oklch_to_rgb(rgb_to_oklch(c)), c);
    }

    /// For a fixed hue and chroma, more OKLCH lightness never gives a darker colour (APCA luminance).
    #[test]
    fn lightness_is_monotonic(h in 0.0f32..360.0, l in 0.05f32..0.9, step in 0.02f32..0.1) {
        let c = 0.04;
        let y = |l| studyfrog_ui::color_science::apca_y(oklch_to_rgb(lch(l, c, h)));
        prop_assert!(y(l + step) >= y(l));
    }

    /// Greys have no chroma; the lightness of a grey rises with its value.
    #[test]
    fn greys_have_no_chroma(v in 0u8..=254) {
        let a = rgb_to_oklch(Color32::from_gray(v));
        let b = rgb_to_oklch(Color32::from_gray(v + 1));
        prop_assert!(a.c < 1e-4);
        prop_assert!(b.l > a.l);
    }

    /// Swapping text and background flips the sign of APCA (or both are 0), never the order of magnitude.
    #[test]
    fn apca_polarity(a in 0u32..0xffffff, b in 0u32..0xffffff) {
        let (x, y) = (apca_lc(hex(a), hex(b)), apca_lc(hex(b), hex(a)));
        prop_assert!(x * y <= 0.0);
        prop_assert!((-108.5..=106.5).contains(&x));
    }
}
