//! The few text colours a note can use. A note keeps one as the small HTML a reader of the file still understands
//! (`<span style="color:#c0392b">text</span>`); only these exact values are read, anything else stays text.

/// A text colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hue {
    /// Red.
    Red,
    /// Orange.
    Orange,
    /// Green.
    Green,
    /// Blue.
    Blue,
    /// Purple.
    Purple,
    /// Grey.
    Grey,
}

/// Every colour, in the order a menu lists them.
pub const HUES: [Hue; 6] = [
    Hue::Red,
    Hue::Orange,
    Hue::Green,
    Hue::Blue,
    Hue::Purple,
    Hue::Grey,
];

const TABLE: [(Hue, &str, &str); 6] = [
    (Hue::Red, "red", "#c0392b"),
    (Hue::Orange, "orange", "#d35400"),
    (Hue::Green, "green", "#1e8449"),
    (Hue::Blue, "blue", "#2471a3"),
    (Hue::Purple, "purple", "#7d3c98"),
    (Hue::Grey, "grey", "#707b7c"),
];

fn row(h: Hue) -> (Hue, &'static str, &'static str) {
    TABLE[HUES.iter().position(|x| *x == h).unwrap_or(0)]
}

/// The colour's name.
pub fn hue_name(h: Hue) -> &'static str {
    row(h).1
}

/// The colour's value in the file.
pub fn hue_hex(h: Hue) -> &'static str {
    row(h).2
}

/// The colour with this name (`grey` and `gray` both).
pub fn hue_from_name(name: &str) -> Option<Hue> {
    let name = name.trim().to_ascii_lowercase();
    let name = if name == "gray" { "grey" } else { &name };
    TABLE
        .iter()
        .find(|(_, n, _)| *n == name)
        .map(|(h, _, _)| *h)
}

/// The colour with this value in the file, in any letter case.
pub fn hue_from_hex(hex: &str) -> Option<Hue> {
    TABLE
        .iter()
        .find(|(_, _, x)| x.eq_ignore_ascii_case(hex.trim()))
        .map(|(h, _, _)| *h)
}

/// The opening tag of a coloured span.
pub fn span_open(h: Hue) -> String {
    format!("<span style=\"color:{}\">", hue_hex(h))
}

/// The colour an opening tag names, when it is exactly such a span.
pub fn hue_of_tag(tag: &str) -> Option<Hue> {
    let hex = tag
        .trim()
        .strip_prefix("<span style=\"color:")?
        .strip_suffix("\">")?;
    hue_from_hex(hex)
}
