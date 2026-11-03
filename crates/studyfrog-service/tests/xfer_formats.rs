#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0209: every importer says what it reads, its examples are read by the importer, and errors name the layout.

use studyfrog_service::xfer_adapters::{ADAPTERS, OBSIDIAN_LAYOUT, read_source};

#[test]
fn every_example_is_read_by_its_importer_and_has_items() {
    for a in &ADAPTERS {
        assert!(a.layout.len() > 40, "{} says what it expects", a.name);
        if let Some(ex) = a.example {
            let bundle =
                (a.read)(ex.text.as_bytes()).unwrap_or_else(|e| panic!("{}: {e}", ex.file));
            assert!(!bundle.items.is_empty(), "{} has items", ex.file);
            let ext = ex.file.rsplit('.').next().unwrap();
            assert!(
                a.extensions.contains(&ext),
                "{} matches its adapter",
                ex.file
            );
        }
    }
    assert!(OBSIDIAN_LAYOUT.contains("Markdown"));
}

#[test]
fn the_csv_example_keeps_a_quoted_question() {
    let csv = ADAPTERS.iter().find(|a| a.name.starts_with("CSV")).unwrap();
    let b = (csv.read)(csv.example.unwrap().text.as_bytes()).unwrap();
    assert_eq!(b.items.len(), 2);
    assert!(b.items[1].text.contains("\"cheval\""));
}

#[test]
fn an_unreadable_file_names_the_layout_that_was_expected() {
    let dir = std::env::temp_dir().join(format!("sf-formats-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("bad.csv");
    std::fs::write(&file, "only one column\nanother\n").unwrap();
    let err = read_source(&file).unwrap_err();
    assert!(
        err.contains("two columns") && err.contains("expects"),
        "{err}"
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn the_help_page_shows_the_same_layouts_and_examples_as_the_importers() {
    let page = studyfrog_service::help::PAGES
        .iter()
        .find(|p| p.id == "import-formats")
        .expect("the page is registered");
    for a in &ADAPTERS {
        assert!(page.text.contains(a.layout), "{} layout", a.name);
        if let Some(ex) = a.example {
            assert!(
                page.text.contains(ex.text.trim_end()),
                "{} example",
                ex.file
            );
        }
    }
    assert!(page.text.contains(OBSIDIAN_LAYOUT));
}
