//! The six text colours of a note, as each theme shows them. The file keeps one value per colour (`studyfrog_md::hue`);
//! on screen each theme shows it in OKLCH at one lightness and chroma (STORY-0113): lighter on a dark theme, darker on
//! a light one, so every colour reads equally well (APCA, `tests/appearance_apca.rs`).

use eframe::egui::Color32;
use studyfrog_md::hue::{HUES, Hue};

use crate::color_science::{lch, oklch_to_rgb};
use crate::theme::Palette;

/// The OKLCH hue of a note colour (grey has almost no chroma).
fn hue_angle(hue: Hue) -> (f32, f32) {
    match hue {
        Hue::Red => (27.0, 1.0),
        Hue::Orange => (55.0, 1.0),
        Hue::Green => (150.0, 1.0),
        Hue::Blue => (250.0, 1.0),
        Hue::Purple => (305.0, 1.0),
        Hue::Grey => (220.0, 0.15),
    }
}

/// The lightness and chroma of note text on a dark and a light theme.
pub const NOTE_TEXT_LC: [(f32, f32); 2] = [(0.80, 0.13), (0.45, 0.14)];

fn compute(hue: Hue, dark: bool) -> Color32 {
    let (l, c) = NOTE_TEXT_LC[usize::from(!dark)];
    let (h, share) = hue_angle(hue);
    oklch_to_rgb(lch(l, c * share, h))
}

fn table() -> &'static [Color32; 12] {
    static ALL: std::sync::OnceLock<[Color32; 12]> = std::sync::OnceLock::new();
    ALL.get_or_init(|| std::array::from_fn(|i| compute(HUES[i % 6], i < 6)))
}

/// The colour of `hue` on the given theme. With high contrast the text colour is used, so contrast comes first.
pub fn hue_color(hue: Hue, dark: bool, high_contrast: bool, palette: &Palette) -> Color32 {
    if high_contrast {
        return palette.text;
    }
    let i = HUES.iter().position(|h| *h == hue).unwrap_or(0);
    table()[i + if dark { 0 } else { 6 }]
}
