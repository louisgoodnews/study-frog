#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0208: the import dialog reads a CSV file with the columns the user chooses and lists every problem.

mod common;

use common::flow::{opened, send};
use common::views::{assert_all_named, harness};
use egui_kittest::kittest::Queryable;
use studyfrog_ui::library_ui::LibraryMsg;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::transfer_csv::{CsvChange, Role};
use studyfrog_ui::transfer_ui::{Tab, TransferMsg};

fn t(m: TransferMsg) -> Msg {
    Msg::Library(LibraryMsg::Transfer(m))
}

fn scratch(label: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("sf-csv-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn a_file_with_other_columns_is_mapped_previewed_with_its_problems_and_imported() {
    let (_vault, mut model, mut worker) = opened("transfer-csv");
    let dir = scratch("map");
    let file = dir.join("ai.csv");
    std::fs::write(
        &file,
        "no;word;meaning;note\n1;cat;chat;x\n2;;empty;x\n3;dog;chien;y\n",
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
    let csv = model.state.transfer.preview.clone().unwrap().csv.unwrap();
    assert_eq!(csv.sheet.delimiter, ';');
    assert_eq!(csv.sheet.names, ["no", "word", "meaning", "note"]);
    assert_eq!(
        (csv.sheet.map.front, csv.sheet.map.back),
        (1, 2),
        "word and meaning"
    );
    let rows: Vec<_> = csv
        .problems
        .iter()
        .map(|p| (p.row, p.column.as_str()))
        .collect();
    assert_eq!(rows, [(3, "word")]);

    send(
        &mut model,
        &mut worker,
        t(TransferMsg::Csv(CsvChange::Column(Role::Front, Some(3)))),
    );
    let csv = model.state.transfer.preview.clone().unwrap().csv.unwrap();
    assert_eq!(
        csv.sheet.map.front, 3,
        "the choice is kept and the file read again"
    );
    assert!(csv.problems.is_empty());

    send(
        &mut model,
        &mut worker,
        t(TransferMsg::Csv(CsvChange::Column(Role::Stack, Some(0)))),
    );
    send(&mut model, &mut worker, t(TransferMsg::Import));
    assert_eq!(
        model.state.notice.as_ref().unwrap().text,
        "Imported: 3 created, 0 replaced, 0 left out."
    );
}

#[test]
fn a_new_file_forgets_the_choices_made_for_the_last_one() {
    let (_vault, mut model, mut worker) = opened("transfer-csv-reset");
    send(
        &mut model,
        &mut worker,
        t(TransferMsg::Open(Tab::Import, None)),
    );
    send(
        &mut model,
        &mut worker,
        t(TransferMsg::Csv(CsvChange::Delimiter(Some('|')))),
    );
    assert_eq!(model.state.transfer.csv.delimiter, Some('|'));
    send(
        &mut model,
        &mut worker,
        t(TransferMsg::Path("/x/other.csv".into())),
    );
    assert_eq!(model.state.transfer.csv.delimiter, None);
}

#[test]
fn the_dialog_shows_the_settings_and_a_button_for_the_problems() {
    let (_vault, mut model, mut worker) = opened("transfer-csv-view");
    let dir = scratch("view");
    let file = dir.join("bad.csv");
    std::fs::write(&file, "front,back\nok,fine\n,missing\n").unwrap();
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
    let mut h = harness(model);
    h.run_steps(6);
    assert!(h.query_by_label("Delimiter").is_some());
    assert!(
        h.query_by_label("The first row names the columns")
            .is_some()
    );
    assert!(h.query_by_label_contains("1 problem(s)").is_some());
    assert!(h.query_by_label_contains("row 3, column front").is_some());
    h.get_by_label("Copy the problems for the AI").click();
    h.run_steps(2);
    assert_all_named(&h, "csv settings");
}
