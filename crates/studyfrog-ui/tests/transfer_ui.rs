#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The import and export dialog: entry points, the export mode question, the preview, duplicates, a cancel, and the files written.

mod common;

use common::flow::{opened, send};
use common::views::{assert_all_named, harness};
use egui_kittest::kittest::Queryable;
use studyfrog_core::review::{Rating, Review};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_service::xfer_zip::read_zip;
use studyfrog_store::schedule::insert_review;
use studyfrog_ui::library_ui::LibraryMsg;
use studyfrog_ui::modal::{Modal, ModalMsg};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::{Screen, Selection};
use studyfrog_ui::transfer_ui::{Format, Tab, TransferMsg};

fn t(m: TransferMsg) -> Msg {
    Msg::Library(LibraryMsg::Transfer(m))
}

fn scratch(label: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("studyfrog-transfer-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn the_dialog_opens_from_the_stack_page_the_create_dialog_and_settings() {
    let (_v, mut model, mut worker) = opened("transfer-open");
    send(&mut model, &mut worker, Msg::Navigate(Screen::Library));
    send(
        &mut model,
        &mut worker,
        Msg::Select(Selection::Stack("french".into())),
    );
    let mut h = harness(model);
    h.run_steps(4);
    h.get_by_label("More actions").click();
    h.run_steps(4);
    h.get_by_label("Import or export…").click();
    h.run_steps(4);
    assert!(matches!(h.state().0.state.modal, Some(Modal::Transfer)));
    let s = &h.state().0.state.transfer;
    assert_eq!((s.tab, s.scope.as_deref()), (Tab::Export, Some("french")));
    assert_all_named(&h, "export tab");

    let (_v2, mut m2, mut w2) = opened("transfer-open2");
    send(&mut m2, &mut w2, Msg::Modal(ModalMsg::OpenCreate));
    let mut h = harness(m2);
    h.run_steps(8);
    h.get_by_label("More").click();
    h.run_steps(4);
    h.get_by_label("Import or export…").click();
    h.run_steps(4);
    assert!(matches!(h.state().0.state.modal, Some(Modal::Transfer)));
    assert_eq!(h.state().0.state.transfer.tab, Tab::Import);
    assert_all_named(&h, "import tab");

    let (_v3, mut m3, mut w3) = opened("transfer-open3");
    m3.state.app.settings_tab = Some("data".into());
    send(&mut m3, &mut w3, Msg::Navigate(Screen::Settings));
    let mut h = harness(m3);
    h.run_steps(4);
    assert!(h.query_by_label("Export…").is_some() && h.query_by_label("Import…").is_some());
}

#[test]
fn an_export_asks_for_the_mode_and_only_the_advanced_mode_writes_the_review_history() {
    let (_v, mut model, mut worker) = opened("transfer-export");
    let r = Review {
        item: Uid(10),
        rating: Rating::Good,
        at: UnixMs(1_700_000_000_000),
        elapsed_days: 0.0,
    };
    insert_review(&worker.session.as_ref().unwrap().conn, &r).unwrap();
    let out = scratch("export");
    send(
        &mut model,
        &mut worker,
        t(TransferMsg::Open(Tab::Export, None)),
    );
    send(
        &mut model,
        &mut worker,
        t(TransferMsg::Destination(out.display().to_string())),
    );
    send(&mut model, &mut worker, t(TransferMsg::AskMode));
    assert!(
        model.state.transfer.choosing_mode,
        "the mode is asked before anything is written"
    );
    assert_eq!(std::fs::read_dir(&out).unwrap().count(), 0);
    send(&mut model, &mut worker, t(TransferMsg::Export));
    let zips: Vec<_> = std::fs::read_dir(&out).unwrap().flatten().collect();
    assert_eq!(zips.len(), 1);
    let files = read_zip(&std::fs::read(zips[0].path()).unwrap()).unwrap();
    let json = String::from_utf8(
        files
            .iter()
            .find(|(p, _)| p == "export.json")
            .unwrap()
            .1
            .clone(),
    )
    .unwrap();
    assert!(
        !json.contains("\"reviews\""),
        "the default mode has no scheduling"
    );
    assert!(
        model
            .state
            .transfer
            .status
            .as_deref()
            .unwrap()
            .starts_with("Exported to")
    );

    send(&mut model, &mut worker, t(TransferMsg::AskMode));
    send(&mut model, &mut worker, t(TransferMsg::Mode(true)));
    send(
        &mut model,
        &mut worker,
        t(TransferMsg::Scope(Some("french".into()))),
    );
    send(&mut model, &mut worker, t(TransferMsg::Export));
    let newest = std::fs::read_dir(&out)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .find(|p| p.to_string_lossy().contains("french"))
        .unwrap();
    let files = read_zip(&std::fs::read(newest).unwrap()).unwrap();
    let json = String::from_utf8(
        files
            .iter()
            .find(|(p, _)| p == "export.json")
            .unwrap()
            .1
            .clone(),
    )
    .unwrap();
    assert!(
        json.contains("\"reviews\"") && json.contains("\"good\""),
        "advanced includes the review history"
    );
    send(&mut model, &mut worker, t(TransferMsg::Format(Format::Csv)));
    send(&mut model, &mut worker, t(TransferMsg::Export));
    assert!(
        std::fs::read_dir(&out)
            .unwrap()
            .flatten()
            .any(|e| e.path().extension().is_some_and(|x| x == "csv"))
    );
    send(
        &mut model,
        &mut worker,
        t(TransferMsg::Destination("/no/such/folder".into())),
    );
    send(&mut model, &mut worker, t(TransferMsg::Export));
    assert!(
        model
            .state
            .transfer
            .status
            .as_deref()
            .unwrap()
            .contains("choose a folder")
    );
    let _ = std::fs::remove_dir_all(&out);
}
