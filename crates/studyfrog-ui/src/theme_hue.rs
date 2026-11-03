//! The six text colours of a note, as each theme shows them. The file keeps one value per colour (`studyfrog_md::hue`);
//! on screen a light theme gets a darker shade and a dark theme a lighter one so the text stays readable.

use eframe::egui::Color32;
use studyfrog_md::hue::Hue;

use crate::theme::Palette;

const fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

/// The colour of `hue` on the given theme. With high contrast the text colour is used, so contrast comes first.
pub fn hue_color(hue: Hue, dark: bool, high_contrast: bool, palette: &Palette) -> Color32 {
    if high_contrast {
        return palette.text;
    }
    match (dark, hue) {
        (true, Hue::Red) => rgb(255, 123, 107),
        (true, Hue::Orange) => rgb(255, 160, 77),
        (true, Hue::Green) => rgb(95, 208, 138),
        (true, Hue::Blue) => rgb(108, 182, 255),
        (true, Hue::Purple) => rgb(195, 155, 255),
        (true, Hue::Grey) => rgb(169, 180, 184),
        (false, Hue::Red) => rgb(168, 40, 27),
        (false, Hue::Orange) => rgb(150, 60, 0),
        (false, Hue::Green) => rgb(23, 104, 58),
        (false, Hue::Blue) => rgb(27, 90, 138),
        (false, Hue::Purple) => rgb(106, 47, 133),
        (false, Hue::Grey) => rgb(80, 89, 91),
    }
}
