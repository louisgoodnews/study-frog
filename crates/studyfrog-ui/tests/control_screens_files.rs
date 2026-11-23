#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! `stack delete` (destructive, the confirmation the folders screen asks) and `pdf open` (the page
//! count, and the reason a file did not open) on a sample vault (STORY-0124, EPIC-064).

mod common;
mod control_common;

use std::fs;

use control_common::*;
use studyfrog_ui::control_cmd::ControlPolicy;

fn destructive_on() -> ControlPolicy {
    ControlPolicy {
        allow_write: true,
        allow_destructive: true,
    }
}

#[test]
fn stack_delete_needs_the_destructive_flag_and_then_removes_the_stack() {
    let (vault, logs) = (
        sample_vault("ctl-stack-del"),
        temp_dir("ctl-stack-del-logs"),
    );
    let mut refused = engine(&vault.path, &logs.path, policy_default());
    let r = ask(&mut refused, "stack delete french");
    assert!(
        r["error"]
            .as_str()
            .unwrap()
            .contains("destructive commands are switched off"),
        "{r}"
    );
    assert!(vault.path.join("french").exists());
    let mut e = engine(&vault.path, &logs.path, destructive_on());
    let done = ask(&mut e, "stack delete french");
    assert_eq!(done["ok"], true, "{done}");
    assert_eq!(done["result"]["deleted"], "french", "{done}");
    assert!(!vault.path.join("french").exists());
    let again = ask(&mut e, "stack delete french");
    assert_eq!(again["ok"], false, "{again}");
}

#[test]
fn pdf_open_answers_the_page_count_of_a_vault_pdf() {
    let (vault, logs) = (sample_vault("ctl-pdf"), temp_dir("ctl-pdf-logs"));
    write_two_page_pdf(&vault.path.join("lecture.pdf"));
    let mut e = engine(&vault.path, &logs.path, policy_default());
    let r = ask(&mut e, "pdf open lecture.pdf");
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["result"]["open"], true, "{r}");
    assert_eq!(r["result"]["page_count"], 2, "{r}");
    assert!(r["result"]["found_pages"].is_array(), "{r}");
}

#[test]
fn pdf_open_of_a_file_that_is_not_a_pdf_answers_the_reason() {
    let (vault, logs) = (sample_vault("ctl-pdf-bad"), temp_dir("ctl-pdf-bad-logs"));
    fs::write(vault.path.join("notes.pdf"), "plain text, not a PDF").unwrap();
    let mut e = engine(&vault.path, &logs.path, policy_default());
    let r = ask(&mut e, "pdf open notes.pdf");
    assert_eq!(r["ok"], false, "{r}");
    assert!(r["error"].as_str().unwrap().contains("did not open"), "{r}");
}

/// A two-page PDF with one line of text per page.
fn write_two_page_pdf(path: &Path) {
    use lopdf::{Document, Stream, dictionary};
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let font_id = doc.add_object(
        dictionary! {"Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica"},
    );
    let mut kids = Vec::new();
    for text in ["reliability first", "second page"] {
        let content = format!("BT /F1 24 Tf 72 700 Td ({text}) Tj ET");
        let content_id = doc.add_object(Stream::new(dictionary! {}, content.into_bytes()));
        let page = dictionary! {
            "Type" => "Page", "Parent" => pages_id, "Contents" => content_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
            "Resources" => dictionary! {"Font" => dictionary! {"F1" => font_id}},
        };
        kids.push(doc.add_object(page).into());
    }
    doc.objects.insert(
        pages_id,
        dictionary! {"Type" => "Pages", "Kids" => kids, "Count" => 2}.into(),
    );
    let catalog_id = doc.add_object(dictionary! {"Type" => "Catalog", "Pages" => pages_id});
    doc.trailer.set("Root", catalog_id);
    doc.save(path).expect("write the test PDF");
}
