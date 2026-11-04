//! Print-safe color palettes for text, syntax tokens, and structural elements.

use studyfrog_md::highlight::Kind;
use studyfrog_md::hue::Hue;

/// RGB color tuple with components in `0..=255`.
pub type Rgb = [u8; 3];

/// Primary body text color: dark charcoal `#1f2328`.
pub const COLOR_TEXT_BODY: Rgb = [31, 35, 40];

/// Link text color: `#2471a3`.
pub const COLOR_LINK: Rgb = [36, 113, 163];

/// Running header and footer text: `#64748b`.
pub const COLOR_MUTED: Rgb = [100, 116, 139];

/// Subtle dividing line and borders: `#e2e8f0`.
pub const COLOR_BORDER_SUBTLE: Rgb = [226, 232, 240];

/// Thematic horizontal rule: `#cbd5e1`.
pub const COLOR_RULE: Rgb = [203, 213, 225];

/// Blockquote vertical left rule: `#bdc3c7`.
pub const COLOR_QUOTE_BAR: Rgb = [189, 195, 199];

/// Code block and table zebra background: `#f8fafc`.
pub const COLOR_BG_CODE: Rgb = [248, 250, 252];

/// Table header background: `#f1f5f9`.
pub const COLOR_BG_TABLE_HEADER: Rgb = [241, 245, 249];

/// Inline code span background pill: `#f1f3f5`.
pub const COLOR_BG_INLINE_CODE: Rgb = [241, 243, 245];

/// Map a markdown [`Hue`] to its WCAG AA print-safe RGB value.
pub fn hue_to_rgb(hue: Hue) -> Rgb {
    match hue {
        Hue::Red => [192, 57, 43],     // #c0392b
        Hue::Orange => [211, 84, 0],   // #d35400
        Hue::Green => [30, 132, 73],   // #1e8449
        Hue::Blue => [36, 113, 163],   // #2471a3
        Hue::Purple => [125, 60, 152], // #7d3c98
        Hue::Grey => [89, 98, 100],    // #596264
    }
}

/// Map a code token syntax [`Kind`] to its highlight RGB color.
pub fn token_kind_color(kind: Kind) -> Rgb {
    match kind {
        Kind::Keyword => [5, 80, 174],    // #0550ae
        Kind::String => [10, 122, 60],    // #0a7a3c
        Kind::Comment => [110, 119, 129], // #6e7781
        Kind::Number => [5, 80, 174],     // #0550ae
        Kind::Plain => COLOR_TEXT_BODY,
    }
}

/// Convert an RGB color into normalized `(r, g, b)` floats in `0.0..=1.0`.
pub fn rgb_to_floats(color: Rgb) -> (f32, f32, f32) {
    (
        f32::from(color[0]) / 255.0,
        f32::from(color[1]) / 255.0,
        f32::from(color[2]) / 255.0,
    )
}
