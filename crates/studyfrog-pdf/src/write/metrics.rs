//! Text metrics and character width lookup for standard PDF typefaces.

/// Standard glyph width in 1/1000th units of font size for Helvetica regular (ASCII 32..=126).
static HELVETICA_ASCII: [u16; 95] = [
    278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278, // 32..47
    556, 556, 556, 556, 556, 556, 556, 556, 556, 556, 278, 278, 584, 584, 584, 556, // 48..63
    1015, 667, 667, 722, 722, 667, 611, 778, 722, 278, 500, 667, 556, 833, 722, 778, // 64..79
    667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611, 278, 278, 278, 469, 556, // 80..95
    222, 556, 556, 500, 556, 556, 278, 556, 556, 222, 222, 500, 222, 833, 556, 556, // 96..111
    556, 556, 333, 500, 278, 556, 500, 722, 500, 500, 500, 334, 260, 334, 584, // 112..126
];

/// Standard glyph width for Helvetica Bold (ASCII 32..=126).
static HELVETICA_BOLD_ASCII: [u16; 95] = [
    278, 333, 474, 556, 556, 889, 722, 238, 333, 333, 389, 584, 278, 333, 278, 278, // 32..47
    556, 556, 556, 556, 556, 556, 556, 556, 556, 556, 333, 333, 584, 584, 584, 611, // 48..63
    975, 722, 722, 722, 722, 667, 611, 778, 722, 278, 556, 722, 611, 889, 722, 778, // 64..79
    722, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611, 333, 278, 333, 584, 556, // 80..95
    333, 556, 611, 556, 611, 556, 333, 611, 611, 278, 278, 556, 278, 889, 611, 611, // 96..111
    611, 611, 389, 556, 333, 611, 556, 778, 556, 556, 500, 389, 280, 389, 584, // 112..126
];

/// Character width in thousandths of an em.
pub fn char_advance_thousandths(c: char, bold: bool, monospace: bool) -> u16 {
    if monospace {
        return 600;
    }
    let code = c as u32;
    if (32..=126).contains(&code) {
        let idx = (code - 32) as usize;
        if bold {
            HELVETICA_BOLD_ASCII[idx]
        } else {
            HELVETICA_ASCII[idx]
        }
    } else {
        match c {
            '•' => 350,
            '—' => 1000,
            '–' => 500,
            '“' | '”' | '‘' | '’' => 333,
            'ä' | 'ö' | 'ü' | 'Ä' | 'Ö' | 'Ü' | 'ß' => {
                if bold {
                    620
                } else {
                    556
                }
            }
            _ => 550,
        }
    }
}

/// Width of a single character in points at the given font size.
pub fn char_width(c: char, size: f32, bold: bool, monospace: bool) -> f32 {
    let units = char_advance_thousandths(c, bold, monospace);
    f32::from(units) * size / 1000.0
}

/// Width of a string slice in points.
pub fn measure_str(text: &str, size: f32, bold: bool, monospace: bool) -> f32 {
    text.chars()
        .map(|c| char_width(c, size, bold, monospace))
        .sum()
}
