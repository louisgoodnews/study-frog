#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Layout and PDF writer tests: golden layouts, proptest width bounds, text round-trip and multi-page stress.

use std::sync::Arc;

use proptest::prelude::*;
use studyfrog_md::blocks::parse_blocks;
use studyfrog_pdf::doc::pdf_info;
use studyfrog_pdf::text::{pdf_all_text, pdf_page_text};
use studyfrog_pdf::write::{
    LayoutHeading, LayoutItem, PageSize, default_layout_options, export_markdown_to_pdf,
    geometry::printable_width, layout_document,
};

#[test]
fn golden_layout_headings_paragraphs_and_rules() {
    let md = "# Title\n\n## Subtitle\n\nA regular paragraph with **bold**, *italic*, and `code`.\n\n---\n\n> A quoted thought.";
    let blocks = parse_blocks(md);
    let opts = default_layout_options();
    let paged = layout_document(&blocks, &opts);

    assert_eq!(paged.pages.len(), 1);
    assert_eq!(paged.title.as_deref(), Some("Title"));
    assert_eq!(paged.headings.len(), 2);
    assert_eq!(paged.headings[0].title, "Title");
    assert_eq!(paged.headings[0].level, 1);
    assert_eq!(paged.headings[1].title, "Subtitle");
    assert_eq!(paged.headings[1].level, 2);

    let has_rule = paged.pages[0]
        .items
        .iter()
        .any(|i| matches!(i, LayoutItem::Rule(_)));
    let has_quote = paged.pages[0]
        .items
        .iter()
        .any(|i| matches!(i, LayoutItem::QuoteBar(_)));
    assert!(has_rule, "horizontal rule laid out");
    assert!(has_quote, "quote bar laid out");
}

#[test]
fn golden_layout_tables_lists_and_tasks() {
    let md = r#"
- [ ] First task
- [x] Completed task
- Normal bullet

1. Ordered item one
2. Ordered item two

| Header A | Header B |
| :--- | :---: |
| Left val | Centered |
"#;
    let blocks = parse_blocks(md);
    let opts = default_layout_options();
    let paged = layout_document(&blocks, &opts);

    assert_eq!(paged.pages.len(), 1);
    let page = &paged.pages[0];

    let checkboxes: Vec<_> = page
        .items
        .iter()
        .filter_map(|i| match i {
            LayoutItem::TaskCheckbox(c) => Some(c.checked),
            _ => None,
        })
        .collect();
    assert_eq!(checkboxes, vec![false, true]);

    let has_table = page.items.iter().any(|i| matches!(i, LayoutItem::Table(_)));
    assert!(has_table, "table laid out");
}

#[test]
fn golden_layout_code_blocks_highlighting() {
    let md = "```rust\nfn main() {\n    let answer = 42;\n}\n```";
    let blocks = parse_blocks(md);
    let opts = default_layout_options();
    let paged = layout_document(&blocks, &opts);

    let code_box = paged.pages[0]
        .items
        .iter()
        .find_map(|i| match i {
            LayoutItem::CodeBox(cb) => Some(cb),
            _ => None,
        })
        .expect("code box item exists");
    assert_eq!(code_box.lang, "rust");
    assert!(!code_box.lines.is_empty());
}

#[test]
fn text_extraction_round_trip_via_pdf_oxide() {
    let md = "# Biology Notes\n\nPhotosynthesis converts light into chemical energy.\n\n- Chloroplasts contain chlorophyll\n- Cellular respiration produces ATP\n";
    let blocks = parse_blocks(md);
    let opts = default_layout_options();

    let pdf_bytes = export_markdown_to_pdf(&blocks, &opts).unwrap();
    let arc_bytes = Arc::new(pdf_bytes);

    let page_text = pdf_page_text(&arc_bytes, 0).unwrap();
    let extracted = &page_text.text;

    for word in [
        "Biology",
        "Notes",
        "Photosynthesis",
        "light",
        "chemical",
        "Chloroplasts",
        "chlorophyll",
        "respiration",
    ] {
        assert!(
            extracted.contains(word),
            "expected word '{word}' in extracted text:\n{extracted}"
        );
    }

    let info = pdf_info(&arc_bytes).unwrap();
    assert_eq!(info.page_count, 1);
    assert_eq!(info.title.as_deref(), Some("Biology Notes"));
}

#[test]
fn multi_page_note_stress_and_structure() {
    let mut md = String::from("# Comprehensive Study Guide\n\n[TOC]\n\n");
    for i in 1..=12 {
        md.push_str(&format!("## Section {i}\n\n"));
        md.push_str("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.\n\n");
        md.push_str("```rust\nfn calculate(x: i32) -> i32 {\n    x * 2 + 1\n}\n```\n\n");
        md.push_str("| Col A | Col B |\n| --- | --- |\n| Value 1 | Value 2 |\n\n");
    }

    let blocks = parse_blocks(&md);
    let mut opts = default_layout_options();
    opts.geometry.size = PageSize::Letter;
    opts.vault_name = Some("MedicineVault".to_string());

    let pdf_bytes = export_markdown_to_pdf(&blocks, &opts).unwrap();
    let arc_bytes = Arc::new(pdf_bytes);

    let info = pdf_info(&arc_bytes).unwrap();
    assert!(
        info.page_count >= 3,
        "multi-page document has {} pages",
        info.page_count
    );

    let all_pages = pdf_all_text(&arc_bytes).unwrap();
    assert_eq!(all_pages.len(), info.page_count);

    let page2 = &all_pages[1].text;
    assert!(
        page2.contains("Page 2 of") || page2.contains("Section"),
        "page 2 has header or footer: {page2}"
    );
}

proptest! {
    #[test]
    fn prop_no_rendered_line_exceeds_printable_width(
        words in prop::collection::vec("[a-zA-Z0-9_]{1,30}", 1..60),
        page_width in 300.0f32..700.0f32
    ) {
        let sentence = words.join(" ");
        let md = format!("# Random Heading\n\n{sentence}\n\n- List item with {sentence}");
        let blocks = parse_blocks(&md);

        let mut opts = default_layout_options();
        opts.geometry.size = PageSize::Custom(page_width, 800.0);
        let allowed_w = printable_width(&opts.geometry);

        let doc = layout_document(&blocks, &opts);
        for page in &doc.pages {
            for item in &page.items {
                match item {
                    LayoutItem::Text(l) | LayoutItem::Heading(LayoutHeading { line: l, .. }) => {
                        let line_w: f32 = l.runs.iter().map(|r| r.width).sum();
                        prop_assert!(
                            line_w <= allowed_w + 1.0,
                            "line width {} exceeds allowed {} (text: {:?})",
                            line_w, allowed_w, l.runs.iter().map(|r| &r.text).collect::<Vec<_>>()
                        );
                    }
                    _ => {}
                }
            }
        }
    }
}
