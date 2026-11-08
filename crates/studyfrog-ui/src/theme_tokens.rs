//! The palettes as perceptual design tokens (STORY-0113, `docs/specs/ui-colour-and-type.md`). Every colour is
//! written in OKLCH: surfaces climb in equal lightness steps, and in each theme all six accents and the status
//! colours share one lightness and one chroma (as much chroma as sRGB allows at that lightness), so switching the
//! accent or reading a row of coloured badges never makes one colour look brighter than the others. The APCA
//! contrast of every pair the theme uses is tested (`tests/appearance_apca.rs`).

use studyfrog_core::appearance::Accent;

use crate::color_science::{Oklch, lch, oklch_to_rgb};
use crate::theme::Palette;

/// The slightly cool hue of the greys.
pub const NEUTRAL_HUE: f32 = 265.0;
/// The hue of `success`.
pub const SUCCESS_HUE: f32 = 150.0;
/// The hue of `warning`.
pub const WARNING_HUE: f32 = 75.0;
/// The hue of `danger`.
pub const DANGER_HUE: f32 = 27.0;
/// The hue of `alt` (the colour that is not the accent).
pub const ALT_HUE: f32 = 300.0;

/// The lightness, chroma and greys of one theme.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tokens {
    /// Wells: text fields, code, the darkest (dark) or most sunken (light) step.
    pub background: Oklch,
    /// Panels and the page.
    pub panel: Oklch,
    /// Cards and popups, one step above the panel.
    pub surface: Oklch,
    /// A surface under the pointer, and quiet button fills.
    pub surface_hover: Oklch,
    /// Hairlines.
    pub border: Oklch,
    /// Body text.
    pub text: Oklch,
    /// Secondary text.
    pub text_weak: Oklch,
    /// Text on an accent fill.
    pub on_accent: Oklch,
    /// The one lightness of the accents and status colours.
    pub accent_l: f32,
    /// Their chroma (less where sRGB cannot show it at that lightness and hue).
    pub accent_c: f32,
}

const fn grey(l: f32, c: f32) -> Oklch {
    lch(l, c, NEUTRAL_HUE)
}

const DARK: Tokens = Tokens {
    background: grey(0.205, 0.008),
    panel: grey(0.235, 0.008),
    surface: grey(0.265, 0.009),
    surface_hover: grey(0.295, 0.010),
    border: grey(0.47, 0.012),
    text: grey(0.93, 0.004),
    text_weak: grey(0.78, 0.012),
    on_accent: lch(0.14, 0.03, NEUTRAL_HUE),
    accent_l: 0.77,
    accent_c: 0.13,
};

const DARK_HIGH: Tokens = Tokens {
    background: grey(0.10, 0.0),
    panel: grey(0.15, 0.004),
    surface: grey(0.20, 0.006),
    surface_hover: grey(0.30, 0.012),
    border: grey(0.74, 0.012),
    text: grey(1.0, 0.0),
    text_weak: grey(0.90, 0.008),
    on_accent: grey(0.0, 0.0),
    accent_l: 0.85,
    accent_c: 0.12,
};

const LIGHT: Tokens = Tokens {
    background: grey(0.950, 0.004),
    panel: grey(0.975, 0.003),
    surface: grey(1.0, 0.0),
    surface_hover: grey(0.925, 0.006),
    border: grey(0.86, 0.010),
    text: grey(0.24, 0.010),
    text_weak: grey(0.52, 0.018),
    on_accent: grey(1.0, 0.0),
    accent_l: 0.48,
    accent_c: 0.15,
};

const LIGHT_HIGH: Tokens = Tokens {
    background: grey(0.94, 0.003),
    panel: grey(0.97, 0.002),
    surface: grey(1.0, 0.0),
    surface_hover: grey(0.92, 0.006),
    border: grey(0.40, 0.014),
    text: grey(0.0, 0.0),
    text_weak: grey(0.33, 0.014),
    on_accent: grey(1.0, 0.0),
    accent_l: 0.42,
    accent_c: 0.16,
};

/// The tokens of a theme.
pub fn tokens(dark: bool, high_contrast: bool) -> Tokens {
    match (dark, high_contrast) {
        (true, false) => DARK,
        (true, true) => DARK_HIGH,
        (false, false) => LIGHT,
        (false, true) => LIGHT_HIGH,
    }
}

/// The OKLCH hue of an accent.
pub fn accent_hue(accent: Accent) -> f32 {
    match accent {
        Accent::Blue => 258.0,
        Accent::Green => 150.0,
        Accent::Purple => 300.0,
        Accent::Orange => 55.0,
        Accent::Pink => 355.0,
        Accent::Teal => 195.0,
    }
}

/// A colour at the theme's accent lightness and chroma in `hue` (chroma lowered to fit sRGB).
pub fn chromatic(t: &Tokens, hue: f32) -> Oklch {
    lch(t.accent_l, t.accent_c, hue)
}

/// The palette of a theme, contrast and accent, computed from the tokens.
pub fn build(dark: bool, high_contrast: bool, accent: Accent) -> Palette {
    let t = tokens(dark, high_contrast);
    let at = |hue| oklch_to_rgb(chromatic(&t, hue));
    Palette {
        background: oklch_to_rgb(t.background),
        panel: oklch_to_rgb(t.panel),
        surface: oklch_to_rgb(t.surface),
        surface_hover: oklch_to_rgb(t.surface_hover),
        border: oklch_to_rgb(t.border),
        text: oklch_to_rgb(t.text),
        text_weak: oklch_to_rgb(t.text_weak),
        accent: at(accent_hue(accent)),
        on_accent: oklch_to_rgb(t.on_accent),
        success: at(SUCCESS_HUE),
        warning: at(WARNING_HUE),
        danger: at(DANGER_HUE),
        alt: at(ALT_HUE),
    }
}
