#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Colours come from the theme, so that both themes, the accents and high contrast reach every screen. A view that
//! writes a colour of its own fails this test, unless it is one of the few meant to look the same everywhere.

use std::fs;
use std::path::{Path, PathBuf};

/// Files that may name colours, and why.
const ALLOWED: [(&str, &str); 8] = [
    ("theme.rs", "the palettes themselves"),
    (
        "color_science.rs",
        "colours computed from OKLCH coordinates",
    ),
    ("view/navbar_logo.rs", "the frog logo has its own green"),
    (
        "view/window_frame.rs",
        "the red close button looks the same in every theme",
    ),
    ("view/pdf_page.rs", "a PDF page is paper: white"),
    (
        "view/pdf_notes.rs",
        "the colours of the user's highlights are data",
    ),
    ("view/pdf_paint.rs", "the highlight overlay"),
    (
        "item_look.rs",
        "an item's colour is data, mixed into a theme surface",
    ),
];

const PATTERNS: [&str; 12] = [
    "Color32::from_rgb",
    "Color32::from_rgba",
    "Color32::from_gray",
    "Color32::WHITE",
    "Color32::BLACK",
    "Color32::RED",
    "Color32::GREEN",
    "Color32::BLUE",
    "Color32::YELLOW",
    "Color32::GRAY",
    "Color32::LIGHT_",
    "Color32::DARK_",
];

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in fs::read_dir(dir).unwrap().filter_map(Result::ok) {
        let p = e.path();
        if p.is_dir() {
            sources(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

#[test]
fn views_take_their_colours_from_the_theme() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    sources(&root, &mut files);
    assert!(files.len() > 100, "found the sources");
    let mut bad = Vec::new();
    for f in files {
        let rel = f
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        if ALLOWED.iter().any(|(a, _)| *a == rel) {
            continue;
        }
        let text = fs::read_to_string(&f).unwrap();
        for (n, line) in text.lines().enumerate() {
            if PATTERNS.iter().any(|p| line.contains(p)) {
                bad.push(format!("{rel}:{}: {}", n + 1, line.trim()));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "colours written into views:\n{}",
        bad.join("\n")
    );
}

#[test]
fn every_allowed_file_exists_and_still_names_a_colour() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for (file, why) in ALLOWED {
        let text = fs::read_to_string(root.join(file)).unwrap_or_else(|_| panic!("{file} is gone"));
        assert!(
            PATTERNS.iter().any(|p| text.contains(p)),
            "{file} no longer names a colour; remove it from the list ({why})"
        );
    }
}
