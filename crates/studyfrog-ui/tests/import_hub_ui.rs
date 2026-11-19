#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Universal Import Hub UI tests: top-bar entry point, format detection, markdown drop to inbox or new stack.

mod common;
mod import_hub_common;

use std::fs;
use std::sync::Arc;

use common::flow::{opened, send};
use common::temp_dir;
use common::views::{assert_all_named, harness};
use egui_kittest::kittest::Queryable;
use import_hub_common::{TestDrop, flush, hub, send_ui, xfer};
use studyfrog_service::xfer_adapters::SourceFormat;
use studyfrog_ui::import_hub::{
    DetectedFormat, HubMsg, ImportDestination, detect_format, format_label,
};
use studyfrog_ui::modal::Modal;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::{Screen, Selection};
use studyfrog_ui::transfer_ui::{Tab, TransferMsg};

#[test]
fn format_detection_covers_all_supported_sources_and_extensions() {
    use DetectedFormat::{BuiltIn as B, Extension as E, Unknown as U};
    use SourceFormat as S;
    let imps = vec![("ext".into(), "Custom cards".into(), vec!["cards".into()])];
    assert_eq!(detect_format("", false, &imps), None);
    let cases = [
        ("a.csv", false, B(S::Csv)),
        ("a.tsv", false, B(S::Csv)),
        ("note.md", false, B(S::Markdown)),
        ("deck.apkg", false, B(S::Anki)),
        ("doc.pdf", false, B(S::Pdf)),
        ("exp.json", false, B(S::Json)),
        ("exp.toml", false, B(S::Toml)),
        ("exp.ron", false, B(S::Ron)),
        ("vault.zip", false, B(S::Zip)),
        ("/tmp/vault", true, B(S::ObsidianFolder)),
        ("custom.cards", false, E("Custom cards".into())),
        ("file.xyz", false, U),
    ];
    for (p, dir, expected) in cases {
        let got = detect_format(p, dir, &imps).unwrap();
        assert_eq!(got, expected, "{p}");
        assert!(!format_label(&got).is_empty());
    }
}

#[test]
fn top_bar_import_is_context_aware_and_drop_imports_markdown_to_inbox_or_new_stack() {
    let (v, mut model, mut worker) = opened("hub-md");
    let dir = temp_dir("hub-ui-md");
    let note = dir.path.join("inbox-note.md");
    fs::write(&note, "# Inbox Note\nDropped markdown content.\n").unwrap();

    send(&mut model, &mut worker, Msg::Navigate(Screen::Library));
    send(
        &mut model,
        &mut worker,
        Msg::Select(Selection::Stack("french".into())),
    );
    let mut h = harness(model);
    h.run_steps(2);
    h.get_by_label("Import").click();
    h.run_steps(4);
    assert!(matches!(h.state().0.state.modal, Some(Modal::Transfer)));
    let tr = &h.state().0.state.transfer;
    assert_eq!(
        (tr.tab, tr.scope.as_deref(), tr.hub.dest_mode),
        (
            Tab::Import,
            Some("french"),
            ImportDestination::ExistingStack
        )
    );

    h.get_by_label("Unstacked vault notes").click();
    h.run_steps(2);
    assert_all_named(&h, "unstacked destination");
    h.input_mut()
        .dropped_files
        .push(Arc::new(TestDrop(note.clone())));
    h.run_steps(4);
    flush(&mut h, &mut worker);
    assert!(
        h.query_by_label("Detected format:").is_some() && h.query_by_label("Markdown").is_some()
    );
    assert!(h.query_by_label_contains("Inbox (vault root)").is_some());
    let import_btns: Vec<_> = h.query_all_by_label("Import").collect();
    // The dialog scrolls, and its import button sits below the viewport: bring it into view.
    import_btns.last().unwrap().scroll_to_me();
    h.run_steps(4);
    h.query_all_by_label("Import").last().unwrap().click();
    h.run_steps(2);
    flush(&mut h, &mut worker);
    assert!(v.path.join("inbox-note.md").is_file());

    send_ui(
        &mut h,
        &mut worker,
        Msg::Modal(studyfrog_ui::modal::ModalMsg::Close),
    );
    send_ui(&mut h, &mut worker, Msg::Select(Selection::Inbox));
    h.get_by_label("Import").click();
    h.run_steps(4);
    assert_eq!(
        h.state().0.state.transfer.hub.dest_mode,
        ImportDestination::UnstackedNotes
    );

    h.get_by_label("New stack").click();
    h.run_steps(2);
    assert!(h.query_by_label("New stack name").is_some());
    assert_all_named(&h, "new stack destination");
    for msg in [
        hub(HubMsg::EditNewStack("Chemistry".into())),
        xfer(TransferMsg::Path(note.display().to_string())),
        xfer(TransferMsg::Preview),
        xfer(TransferMsg::Import),
    ] {
        send_ui(&mut h, &mut worker, msg);
    }
    assert!(
        v.path.join("chemistry/_stack.md").is_file()
            && v.path.join("chemistry/inbox-note.md").is_file()
    );
}
