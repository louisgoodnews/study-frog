#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The import half of the dialog: the preview, a cancel, the duplicate choice and the screen.

mod common;

use common::flow::{opened, send};
use common::views::{assert_all_named, harness};
use egui_kittest::kittest::Queryable;
use studyfrog_service::xfer_plan::Policy;
use studyfrog_ui::library_ui::LibraryMsg;
use studyfrog_ui::modal::ModalMsg;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::transfer_ui::{Tab, TransferMsg};

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
fn an_import_shows_a_preview_writes_nothing_on_cancel_and_follows_the_duplicate_choice() {
    let (vault, mut model, mut worker) = opened("transfer-import");
    let dir = scratch("import");
    let file = dir.join("cards.csv");
    std::fs::write(
        &file,
        "stack,front,back\nfrench,What?,That\nfrench,What?,That\n,no stack,x\n",
    )
    .unwrap();
    send(
        &mut model,
        &mut worker,
        t(TransferMsg::Open(Tab::Import, None)),
    );
    send(
        &mut model,
        &mut worker,
        t(TransferMsg::Path(file.display().to_string())),
    );
    send(&mut model, &mut worker, t(TransferMsg::Preview));
    let p = model.state.transfer.preview.clone().expect("a preview");
    assert_eq!(
        (p.adapter.as_str(), p.total, p.counts),
        ("CSV or TSV cards", 3, (2, 0, 1)),
        "{p:?}"
    );
    let before = std::fs::read_dir(vault.path.join("french"))
        .unwrap()
        .count();
    send(&mut model, &mut worker, Msg::Modal(ModalMsg::Close));
    assert_eq!(
        std::fs::read_dir(vault.path.join("french"))
            .unwrap()
            .count(),
        before,
        "a cancel writes nothing"
    );

    send(&mut model, &mut worker, t(TransferMsg::Import));
    assert_eq!(
        model.state.notice.as_ref().unwrap().text,
        "Imported: 2 created, 0 replaced, 1 left out."
    );
    send(&mut model, &mut worker, t(TransferMsg::Preview));
    assert_eq!(
        model.state.transfer.preview.as_ref().unwrap().counts,
        (0, 2, 1),
        "now both exist"
    );
    send(
        &mut model,
        &mut worker,
        t(TransferMsg::Policy(Policy::KeepBoth)),
    );
    send(&mut model, &mut worker, t(TransferMsg::Import));
    assert!(
        model
            .state
            .notice
            .as_ref()
            .unwrap()
            .text
            .starts_with("Imported: 2 created")
    );
    send(
        &mut model,
        &mut worker,
        t(TransferMsg::Path("/no/such/file.csv".into())),
    );
    send(&mut model, &mut worker, t(TransferMsg::Preview));
    assert!(model.state.transfer.preview.is_none() && model.state.transfer.status.is_some());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_preview_screen_names_its_counts_choices_and_buttons() {
    let (_v, mut model, mut worker) = opened("transfer-view");
    let dir = scratch("view");
    let file = dir.join("cards.csv");
    std::fs::write(&file, "stack,front,back\nfrench,Q,A\n").unwrap();
    send(
        &mut model,
        &mut worker,
        t(TransferMsg::Open(Tab::Import, Some("french".into()))),
    );
    send(
        &mut model,
        &mut worker,
        t(TransferMsg::Path(file.display().to_string())),
    );
    send(&mut model, &mut worker, t(TransferMsg::Preview));
    let mut h = harness(model);
    h.run_steps(5);
    assert!(
        h.query_by_label_contains("1 to create, 0 already exist")
            .is_some()
    );
    for name in [
        "Preview",
        "Import",
        "Choose file…",
        "Choose folder…",
        "Cancel",
    ] {
        assert!(h.query_all_by_label(name).count() >= 1, "{name}");
    }
    assert_all_named(&h, "import preview");
    h.get_by_label("Cancel").click();
    h.run_steps(3);
    assert!(h.state().0.state.modal.is_none());
    let _ = std::fs::remove_dir_all(&dir);
}
