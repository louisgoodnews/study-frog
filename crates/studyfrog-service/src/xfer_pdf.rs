//! The PDF reader: a PDF becomes one card per page that has text. The page's first line is the
//! front, the lines below it the back, and the page's number the title. Scanned pages and pages
//! with a single line have no card and are reported in the preview.

use std::sync::Arc;

use studyfrog_pdf::text::pdf_all_text;

use studyfrog_core::plan::ItemKind;

use crate::xfer_model::{Bundle, XItem, item};

/// The adapter's name, which the title of each card is prefixed with the file name for.
pub const PDF_NAME: &str = "PDF document (one card per page)";

/// Longest front, in characters.
const FRONT_MAX: usize = 200;
/// Longest back, in characters.
const BACK_MAX: usize = 2000;

fn clip(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

/// The cards for pages given as (page number from 1, text). Pure.
pub fn page_cards(pages: &[(usize, &str)]) -> (Vec<XItem>, Vec<String>) {
    let mut items = Vec::new();
    let mut warnings = Vec::new();
    for &(number, text) in pages {
        let lines: Vec<&str> = text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect();
        if lines.len() < 2 {
            warnings.push(format!("page {number} has too little text for a card"));
            continue;
        }
        let mut card = item(
            ItemKind::Card,
            "",
            &clip(lines[0], FRONT_MAX),
            &clip(&lines[1..].join("\n"), BACK_MAX),
        );
        card.title = format!("Page {number}");
        items.push(card);
    }
    (items, warnings)
}

/// Read a PDF file's bytes into cards, one per page with text.
pub fn from_pdf(bytes: &[u8]) -> Result<Bundle, String> {
    let pages = pdf_all_text(&Arc::new(bytes.to_vec()))
        .map_err(|e| studyfrog_pdf::error::describe_pdf_error(&e))?;
    let texts: Vec<(usize, &str)> = pages
        .iter()
        .enumerate()
        .map(|(i, p)| (i + 1, p.text.as_str()))
        .collect();
    let (items, warnings) = page_cards(&texts);
    Ok(Bundle {
        items,
        warnings,
        ..Bundle::default()
    })
}

/// Put the file's name in front of each card's title, so cards from different PDFs stay apart.
pub fn name_cards(bundle: &mut Bundle, file: &str) {
    let stem = std::path::Path::new(file)
        .file_stem()
        .map_or_else(|| file.to_owned(), |s| s.to_string_lossy().into_owned());
    for item in &mut bundle.items {
        item.title = format!("{stem}: {}", item.title);
    }
}
