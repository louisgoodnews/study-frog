#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Importing and making cards through the executor and the library screen.

mod common;

use std::fs;

use common::ext::{model_in_stack, registry};
use common::views::{assert_all_named, harness, open_model};
use common::{NOW_NS, TempDir, sample_vault, temp_dir};
use egui_kittest::kittest::Queryable;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{Worker, execute, worker_with_host};
use studyfrog_ui::host::Host;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Selection;

fn worker_with_registry() -> (TempDir, Worker) {
    let vault = sample_vault("import");
    let mut w = worker_with_host(
        None,
        None,
        Host {
            registry: registry(),
            ..Host::default()
        },
    );
    execute(
        &mut w,
        Effect::OpenVault(vault.path.display().to_string()),
        NOW_NS,
    );
    (vault, w)
}

#[test]
fn the_executor_imports_a_file_with_the_importer_for_its_extension() {
    let (_vault, mut w) = worker_with_registry();
    let dir = temp_dir("import-files");
    let good = dir.path.join("deck.CSV");
    fs::write(&good, "Hola,Hello\nAdios,Bye\n").unwrap();
    let msgs = execute(
        &mut w,
        Effect::ImportFile {
            folder: "french".into(),
            path: good.display().to_string(),
        },
        NOW_NS,
    );
    let Msg::Imported(Ok(out)) = &msgs[0] else {
        panic!("{:?}", msgs[0])
    };
    assert_eq!(out.report.created, 2);
    assert_eq!(
        out.info.overview.stacks[0].cards, 4,
        "the sample stack had two cards"
    );

    let broken = dir.path.join("bad.csv");
    fs::write(&broken, "fine,ok\nno comma here\n").unwrap();
    let msgs = execute(
        &mut w,
        Effect::ImportFile {
            folder: "french".into(),
            path: broken.display().to_string(),
        },
        NOW_NS,
    );
    assert!(
        matches!(&msgs[0], Msg::Imported(Err(e)) if e.contains("Comma lines") && e.contains("line 2")),
        "{:?}",
        msgs[0]
    );

    let other = dir.path.join("deck.pdf");
    fs::write(&other, "x").unwrap();
    let msgs = execute(
        &mut w,
        Effect::ImportFile {
            folder: "french".into(),
            path: other.display().to_string(),
        },
        NOW_NS,
    );
    assert!(
        matches!(&msgs[0], Msg::Imported(Err(e)) if e.contains("known: csv")),
        "{:?}",
        msgs[0]
    );
    let msgs = execute(
        &mut w,
        Effect::ImportFile {
            folder: "french".into(),
            path: "/nonexistent/deck.csv".into(),
        },
        NOW_NS,
    );
    assert!(matches!(&msgs[0], Msg::Imported(Err(_))));
}

#[test]
fn the_executor_expands_a_card_type_and_explains_its_refusals() {
    let (_vault, mut w) = worker_with_registry();
    let make = |source: &str, type_id: &str| Effect::CreateFromType {
        folder: "french".into(),
        type_id: type_id.into(),
        source: source.into(),
    };
    let msgs = execute(&mut w, make("bonjour", "twice"), NOW_NS);
    let Msg::Imported(Ok(out)) = &msgs[0] else {
        panic!("{:?}", msgs[0])
    };
    assert_eq!(out.report.created, 2);
    let msgs = execute(&mut w, make("no!", "twice"), NOW_NS);
    assert!(
        matches!(&msgs[0], Msg::Imported(Err(e)) if e.contains("Two cards") && e.contains("exclamation"))
    );
    let msgs = execute(&mut w, make("x", "missing"), NOW_NS);
    assert!(matches!(&msgs[0], Msg::Imported(Err(e)) if e.contains("not available")));
}

#[test]
fn the_import_dialog_offers_card_types_and_files_only_when_extensions_exist() {
    let mut h = harness(model_in_stack());
    h.run();
    assert!(
        h.query_by_label("Card front").is_none(),
        "the page holds no card form any more"
    );
    h.get_by_label("Import…").click();
    h.run();
    assert_all_named(&h, "import dialog");
    for text in [
        "Card type:",
        "Two cards",
        "Import cards",
        "Reads: Comma lines",
        "File to import",
        "Browse…",
    ] {
        assert!(h.query_by_label(text).is_some(), "missing {text}");
    }
    h.get_by_label("Two cards").click();
    h.run();
    assert_eq!(
        h.state().0.state.browser.card_type.as_deref(),
        Some("twice")
    );
    assert!(
        h.query_by_label("Source text").is_some() && h.query_by_label("Write a word.").is_some()
    );
    assert!(h.query_by_label("Add cards").is_some());

    let mut plain = open_model();
    apply(&mut plain, Msg::Select(Selection::Stack("french".into())));
    let mut h = harness(plain);
    h.run();
    assert!(
        h.query_by_label("Import…").is_none(),
        "without extensions there is nothing to import with"
    );
}
