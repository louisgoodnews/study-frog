#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests verifying PDF export of the comprehensive documentation example note.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use studyfrog_md::ulid::parse_ulid;
use studyfrog_pdf::doc::pdf_info;
use studyfrog_pdf::text::pdf_all_text;
use studyfrog_service::pdf_export::{ExportTarget, default_pdf_export_options, export_pdf};
use studyfrog_service::session::{VaultSession, open_vault};
use studyfrog_store::reindex::reindex;

struct TempDir(PathBuf);
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn source_example_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pdf-export/complete-note.md")
}

fn create_temp_vault(label: &str) -> (TempDir, VaultSession) {
    let dir =
        std::env::temp_dir().join(format!("studyfrog-pdf-docs-{label}-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let (s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    (TempDir(dir), s)
}

fn write_file(root: &Path, rel: &str, content: &[u8]) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, content).unwrap();
}

fn extract_full_text(pdf_bytes: &[u8]) -> String {
    let pages = pdf_all_text(&Arc::new(pdf_bytes.to_vec())).unwrap();
    pages
        .into_iter()
        .map(|p| p.text)
        .collect::<Vec<_>>()
        .join("\n")
}

fn verify_headings(extracted: &str) {
    const HEADINGS: &[&str] = &[
        "Complete PDF Export Showcase",
        "Typography and Text Styling",
        "The Six Print-Safe Palette Hues",
        "Headings and Document Structure",
        "Major Subsection",
        "Minor Technical Detail",
        "Lists and Task Tracking",
        "Blockquotes and Citations",
        "Tables with Column Alignment",
        "Syntax-Highlighted Code Blocks",
        "Links and Cross References",
        "Image Attachments",
    ];
    for h in HEADINGS {
        assert!(extracted.contains(h), "Missing heading '{h}'");
    }
}

fn verify_styles_and_lists(extracted: &str) {
    const PHRASES: &[&str] = &[
        "Bold typography",
        "Italic styling",
        "Underlined text passages",
        "Red text",
        "Orange text",
        "Green text",
        "Blue text",
        "Purple text",
        "Grey text",
        "Fundamental concept of memory consolidation",
        "Sequential Numbered Steps",
        "Establish foundational vault taxonomy",
        "Conduct comprehensive end-of-term knowledge review",
    ];
    for p in PHRASES {
        assert!(extracted.contains(p), "Missing phrase '{p}'");
    }
}

fn verify_content_blocks(extracted: &str) {
    const TOKENS: &[&str] = &[
        "Active recall and spaced repetition",
        "Principles of Human Memory",
        "Molecular Genetics",
        "Discrete Structures",
        "Quantum Mechanics",
        "Cognitive Science",
        "92.4%",
        "Urgent",
        "compute_stability",
        "spaced_intervals",
        "StudyFrog Source Repository",
        "Cognitive Architecture Overview",
    ];
    for t in TOKENS {
        assert!(extracted.contains(t), "Missing token '{t}'");
    }
    assert!(
        extracted.contains("Cellular Biology Overview Diagram"),
        "Missing picture caption in extracted text"
    );
}

fn verify_outlines(pdf_bytes: &[u8]) {
    let info = pdf_info(&Arc::new(pdf_bytes.to_vec())).unwrap();
    assert!(info.page_count >= 2, "Expected multiple pages with cover");
    assert!(
        info.outline
            .iter()
            .any(|e| e.title == "Typography and Text Styling"),
        "Outline should contain main headings"
    );
    assert!(
        info.outline.iter().any(|e| e.title == "Major Subsection"),
        "Outline should contain subsection headings"
    );
}

#[test]
fn test_export_complete_documentation_example_and_verify_all_elements() {
    let example_path = source_example_path();
    assert!(
        example_path.is_file(),
        "example file must exist at {example_path:?}"
    );

    let example_content = fs::read_to_string(&example_path).unwrap();
    let (temp_dir, mut session) = create_temp_vault("docs_example");

    write_file(&temp_dir.0, "complete-note.md", example_content.as_bytes());
    let target_uid = parse_ulid("01HZY000000000000000000001").unwrap();

    let cognitive_md = "---\nid: 01HZY000000000000000000002\nkind: note\ntitle: Cognitive Architecture Overview\n---\n# Cognitive Architecture Overview\n";
    write_file(&temp_dir.0, "cognitive.md", cognitive_md.as_bytes());
    write_file(
        &temp_dir.0,
        "assets/cellular_biology_overview.png",
        b"fake_png",
    );

    reindex(&mut session.conn, &session.root).unwrap();

    let output_pdf = temp_dir.0.join("exported_showcase.pdf");
    let mut opts = default_pdf_export_options();
    opts.table_of_contents = true;
    opts.title_page = true;

    let result = export_pdf(
        &session,
        &ExportTarget::SingleItem(target_uid),
        &opts,
        &output_pdf,
    )
    .unwrap();
    let target_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target");
    if target_dir.is_dir() {
        let _ = fs::copy(&output_pdf, target_dir.join("complete-note-export.pdf"));
    }

    assert_eq!(result.item_count, 1);
    assert!(output_pdf.exists());
    assert!(result.bytes_written > 1000);

    let pdf_bytes = fs::read(&output_pdf).unwrap();
    assert!(pdf_bytes.starts_with(b"%PDF"));

    let extracted = extract_full_text(&pdf_bytes);
    verify_headings(&extracted);
    verify_styles_and_lists(&extracted);
    verify_content_blocks(&extracted);
    verify_outlines(&pdf_bytes);
}
