//! Item colours: an optional `color` in an item's front matter, either a word from a small palette
//! (`sage`, `sky`, ...) or a `#RRGGBB` code. Pure: reading and writing the value; how a colour tints
//! a surface is the interface's business.

/// The front-matter key that holds an item's colour.
pub const COLOR_FIELD: &str = "color";

/// The palette offered by the colour picker: the stored word, the word shown and the colour.
pub const PALETTE: [(&str, &str, [u8; 3]); 7] = [
    ("sage", "Sage", [0x7B, 0xB0, 0x7E]),
    ("sky", "Sky blue", [0x5B, 0x9B, 0xD5]),
    ("rose", "Rose", [0xD9, 0x6C, 0x86]),
    ("lavender", "Lavender", [0x9B, 0x86, 0xD6]),
    ("amber", "Amber", [0xE0, 0xA4, 0x3A]),
    ("teal", "Teal", [0x3F, 0xA8, 0xA0]),
    ("slate", "Slate", [0x7A, 0x88, 0x99]),
];

/// Other words a hand-written file may use for a palette colour.
const ALIASES: [(&str, &str); 5] = [
    ("green", "sage"),
    ("blue", "sky"),
    ("purple", "lavender"),
    ("red", "rose"),
    ("gray", "slate"),
];

fn palette_word(word: &str) -> Option<&'static str> {
    let lower = word.to_ascii_lowercase();
    let alias = ALIASES.iter().find(|(a, _)| *a == lower).map(|(_, w)| *w);
    let word = alias.unwrap_or(lower.as_str());
    PALETTE
        .iter()
        .find(|(w, _, _)| *w == word)
        .map(|(w, _, _)| *w)
}

fn hex_rgb(text: &str) -> Option<[u8; 3]> {
    let digits = text.strip_prefix('#')?;
    if digits.len() != 6 || !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let byte = |i: usize| u8::from_str_radix(&digits[i..i + 2], 16).ok();
    Some([byte(0)?, byte(2)?, byte(4)?])
}

/// The colour a stored value names; `None` for anything else (an unknown word, a short code).
pub fn color_rgb(value: &str) -> Option<[u8; 3]> {
    let value = value.trim().trim_matches(|c| c == '"' || c == '\'');
    if let Some(word) = palette_word(value) {
        return PALETTE
            .iter()
            .find(|(w, _, _)| *w == word)
            .map(|(_, _, c)| *c);
    }
    hex_rgb(value)
}

/// The value in the one form it is stored in: a palette word in lower case or `#RRGGBB` in upper
/// case; `None` for a value that names no colour.
pub fn normalize_color(value: &str) -> Option<String> {
    let value = value.trim().trim_matches(|c| c == '"' || c == '\'');
    if let Some(word) = palette_word(value) {
        return Some(word.to_owned());
    }
    hex_rgb(value).map(|[r, g, b]| format!("#{r:02X}{g:02X}{b:02X}"))
}

/// The value as front matter writes it: a code is quoted, because `#` starts a YAML comment.
pub fn color_front_matter(value: &str) -> Option<String> {
    let v = normalize_color(value)?;
    Some(if v.starts_with('#') {
        format!("\"{v}\"")
    } else {
        v
    })
}

/// The `#RRGGBB` code of a colour.
pub fn color_hex([r, g, b]: [u8; 3]) -> String {
    format!("#{r:02X}{g:02X}{b:02X}")
}
