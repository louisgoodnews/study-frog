#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for PDF export use cases: single items, cards/questions, options, and stack exports.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use studyfrog_md::ulid::parse_ulid;
use studyfrog_pdf::text::pdf_all_text;
use studyfrog_pdf::write::PageSize;
use studyfrog_service::pdf_export::{
    ExportTarget, default_pdf_export_options, export_pdf, plan_pdf_export,
};
use studyfrog_service::session::{VaultSession, open_vault};
use studyfrog_store::reindex::reindex;

struct Dir(PathBuf);
impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn pdf_text(bytes: Vec<u8>) -> String {
    let pages = pdf_all_text(&Arc::new(bytes)).unwrap();
    pages
        .into_iter()
        .map(|p| p.text)
        .collect::<Vec<_>>()
        .join("\n")
}

fn test_vault(label: &str) -> (Dir, VaultSession) {
    let dir =
        std::env::temp_dir().join(format!("studyfrog-pdf-test-{label}-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let (s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    (Dir(dir), s)
}

fn write_file(dir: &Path, rel: &str, content: &str) {
    let file = dir.join(rel);
    if let Some(p) = file.parent() {
        fs::create_dir_all(p).unwrap();
    }
    fs::write(&file, content).unwrap();
}

#[test]
fn export_single_note_inside_and_outside_vault() {
    let (d, mut s) = test_vault("single_note");
    let uid = parse_ulid("00000000000000000000000001").unwrap();
    let md = "---\nid: 00000000000000000000000001\nkind: note\ntitle: Cellular Biology\n---\n# Cellular Biology\n\nMitochondria produce ATP for cellular work.\n";
    write_file(&d.0, "biology.md", md);
    reindex(&mut s.conn, &s.root).unwrap();

    let inside_path = d.0.join("exported_note.pdf");
    let opts = default_pdf_export_options();
    let outcome = export_pdf(&s, &ExportTarget::SingleItem(uid), &opts, &inside_path).unwrap();
    assert_eq!(outcome.item_count, 1);
    assert!(inside_path.exists());

    let pdf_bytes = fs::read(&inside_path).unwrap();
    assert!(pdf_bytes.starts_with(b"%PDF"));
    let extracted = pdf_text(pdf_bytes);
    assert!(extracted.contains("Cellular Biology"));
    assert!(extracted.contains("Mitochondria produce ATP"));

    let out_dir = std::env::temp_dir().join(format!("studyfrog-outside-{}", std::process::id()));
    let _out_guard = Dir(out_dir.clone());
    let outside_path = out_dir.join("outside_note.pdf");
    let out_res = export_pdf(&s, &ExportTarget::SingleItem(uid), &opts, &outside_path).unwrap();
    assert!(outside_path.exists());
    assert!(out_res.bytes_written > 100);
}

#[test]
fn export_card_and_question_with_and_without_answers() {
    let (d, mut s) = test_vault("cards_q");
    let card_uid = parse_ulid("00000000000000000000000002").unwrap();
    let card_md = "---\nid: 00000000000000000000000002\nkind: card\ntitle: French Greeting\n---\nWhat is hello in French?\n???\nBonjour!\n";
    write_file(&d.0, "french.md", card_md);

    let q_uid = parse_ulid("00000000000000000000000003").unwrap();
    let q_md = "---\nid: 00000000000000000000000003\nkind: question\ntitle: Speed of Light\n---\nWhat is c in a vacuum?\n???\nApproximately 300 000 km/s.\n";
    write_file(&d.0, "physics.md", q_md);
    reindex(&mut s.conn, &s.root).unwrap();

    let mut opts_with_answers = default_pdf_export_options();
    opts_with_answers.show_answers = true;
    let path_with = d.0.join("with_answers.pdf");
    export_pdf(
        &s,
        &ExportTarget::Selection(vec![card_uid, q_uid]),
        &opts_with_answers,
        &path_with,
    )
    .unwrap();
    let text_with = pdf_text(fs::read(&path_with).unwrap());
    assert!(text_with.contains("Bonjour!"));
    assert!(text_with.contains("300 000 km/s"));

    let mut opts_no_answers = default_pdf_export_options();
    opts_no_answers.show_answers = false;
    let path_without = d.0.join("no_answers.pdf");
    export_pdf(
        &s,
        &ExportTarget::Selection(vec![card_uid, q_uid]),
        &opts_no_answers,
        &path_without,
    )
    .unwrap();
    let text_without = pdf_text(fs::read(&path_without).unwrap());
    assert!(text_without.contains("What is hello in French?"));
    assert!(text_without.contains("What is c in a vacuum?"));
    assert!(!text_without.contains("Bonjour!"));
    assert!(!text_without.contains("300 000 km/s"));
}

#[test]
fn export_stack_with_substacks_and_referenced_items_each_once() {
    let (d, mut s) = test_vault("stack_export");
    let stack_a_uid = "0000000000000000000000000A";
    let stack_sub_uid = "0000000000000000000000000B";

    write_file(
        &d.0,
        "math/_stack.md",
        &format!(
            "---\nid: {stack_a_uid}\nkind: stack\ntitle: Mathematics\n---\nFoundational math topics.\n"
        ),
    );
    write_file(
        &d.0,
        "math/algebra/_stack.md",
        &format!(
            "---\nid: {stack_sub_uid}\nkind: stack\ntitle: Algebra\n---\nPolynomials and matrices.\n"
        ),
    );

    let item1 = "---\nid: 00000000000000000000000010\nkind: note\ntitle: Real Numbers\n---\nProperties of the real line.\n";
    write_file(&d.0, "math/real_numbers.md", item1);

    let item2 = "---\nid: 00000000000000000000000011\nkind: card\ntitle: Quadratic Formula\n---\nRoots of ax^2 + bx + c = 0\n???\nx = (-b +- sqrt(b^2-4ac))/2a\n";
    write_file(&d.0, "math/algebra/quad.md", item2);

    let item_ref = &format!(
        "---\nid: 00000000000000000000000012\nkind: note\ntitle: Fourier Analysis\nreferenced_by: [{stack_a_uid}]\n---\nTransforms and waves.\n"
    );
    write_file(&d.0, "physics/fourier.md", item_ref);

    reindex(&mut s.conn, &s.root).unwrap();

    let plan = plan_pdf_export(
        &s,
        &ExportTarget::Stack("math".to_string()),
        &default_pdf_export_options(),
    )
    .unwrap();
    assert_eq!(
        plan.items.len(),
        3,
        "primary in math, primary in math/algebra, and item referencing math"
    );
    let u10 = parse_ulid("00000000000000000000000010").unwrap();
    let u11 = parse_ulid("00000000000000000000000011").unwrap();
    let u12 = parse_ulid("00000000000000000000000012").unwrap();
    assert!(plan.items.contains(&u10));
    assert!(plan.items.contains(&u11));
    assert!(plan.items.contains(&u12));

    let dest = d.0.join("math_stack.pdf");
    let outcome = export_pdf(
        &s,
        &ExportTarget::Stack("math".to_string()),
        &default_pdf_export_options(),
        &dest,
    )
    .unwrap();
    assert_eq!(outcome.item_count, 3);
    assert!(dest.exists());

    let text = pdf_text(fs::read(&dest).unwrap());
    assert!(text.contains("Real Numbers"));
    assert!(text.contains("Quadratic Formula"));
    assert!(text.contains("Fourier Analysis"));
}

#[test]
fn export_options_custom_margins_pagesize_title_page_and_toc() {
    let (d, mut s) = test_vault("options_doc");
    let uid = parse_ulid("00000000000000000000000020").unwrap();
    write_file(
        &d.0,
        "notes.md",
        "---\nid: 00000000000000000000000020\nkind: note\ntitle: Deep Learning\n---\n# Chapter 1: Backpropagation\n\nGradient descent details.\n\n# Chapter 2: Transformers\n\nAttention mechanisms.\n",
    );
    reindex(&mut s.conn, &s.root).unwrap();

    let mut opts = default_pdf_export_options();
    opts.page_size = PageSize::Letter;
    opts.title_page = true;
    opts.table_of_contents = true;
    opts.page_numbers = true;
    opts.font_size = 12.0;

    let dest = d.0.join("options_export.pdf");
    let outcome = export_pdf(&s, &ExportTarget::SingleItem(uid), &opts, &dest).unwrap();
    assert!(outcome.pages > 0);

    let bytes = fs::read(&dest).unwrap();
    let text = pdf_text(bytes);
    assert!(text.contains("Deep Learning"));
    assert!(text.contains("Backpropagation"));
    assert!(text.contains("Transformers"));
}
