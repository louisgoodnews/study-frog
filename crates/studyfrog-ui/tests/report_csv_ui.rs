#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Exporting the reports as CSV: the button, and the file the worker writes into the vault.

mod common;

use common::views::{assert_all_named, harness, open_model};
use common::{NOW_NS, sample_vault};
use egui_kittest::kittest::Queryable;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{execute, worker_new};
use studyfrog_ui::library_bulk::LibraryEffect;
use studyfrog_ui::library_ui::LibraryMsg;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;
use studyfrog_ui::transfer_types::TransferJob;
use studyfrog_ui::transfer_ui::TransferMsg;

#[test]
fn the_reports_screen_has_an_export_button_that_asks_for_the_csv() {
    let mut m = open_model();
    m.state.screen = Screen::Reports;
    let mut h = harness(m);
    h.run();
    h.get_by_label("Export as CSV").click();
    h.run_steps(3);
    assert!(
        h.state()
            .1
            .contains(&Effect::Library(LibraryEffect::Transfer(
                TransferJob::ReportCsv(false)
            )))
    );
    assert_all_named(&h, "reports");
}

#[test]
fn the_worker_writes_the_csv_into_the_exports_folder_of_the_vault() {
    let vault = sample_vault("report-csv");
    let mut worker = worker_new(None);
    execute(
        &mut worker,
        Effect::OpenVault(vault.path.display().to_string()),
        NOW_NS,
    );
    let job = Effect::Library(LibraryEffect::Transfer(TransferJob::ReportCsv(false)));
    let msgs = execute(&mut worker, job, NOW_NS);
    let Msg::Library(LibraryMsg::Transfer(TransferMsg::Exported(Ok(path)))) = &msgs[0] else {
        panic!("{:?}", msgs[0]);
    };
    assert!(path.contains("/exports/studyfrog-report-"), "{path}");
    assert!(path.ends_with(".csv"), "{path}");
    let text = std::fs::read_to_string(path).unwrap();
    let rows: Vec<&str> = text.split("\r\n").filter(|l| !l.is_empty()).collect();
    assert_eq!(rows[0], "section,subject,metric,value");
    assert!(
        rows.iter().any(|r| r.starts_with("summary,all,reviews,")),
        "{rows:#?}"
    );
    assert!(
        rows.iter().any(|r| r.starts_with("forecast,day 0,")),
        "{rows:#?}"
    );
    assert!(rows.iter().all(|r| r.split(',').count() >= 4), "{rows:#?}");
}

#[test]
fn without_a_vault_nothing_is_written_and_the_answer_says_why() {
    let mut worker = worker_new(None);
    let job = Effect::Library(LibraryEffect::Transfer(TransferJob::ReportCsv(false)));
    let msgs = execute(&mut worker, job, NOW_NS);
    assert_eq!(msgs.len(), 1);
    assert!(
        format!("{:?}", msgs[0]).contains("no vault is open"),
        "{:?}",
        msgs[0]
    );
}

fn chosen_folder() -> Option<std::path::PathBuf> {
    let dir = std::env::temp_dir().join(format!("sf-report-chosen-{}", std::process::id()));
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

fn cancelled() -> Option<std::path::PathBuf> {
    None
}

#[test]
fn save_to_asks_for_a_folder_and_writes_the_stacks_and_totals_there() {
    let mut m = open_model();
    m.state.screen = Screen::Reports;
    let mut h = harness(m);
    h.run();
    h.get_by_label("Save CSV to...").click();
    h.run_steps(3);
    let to = Effect::Library(LibraryEffect::Transfer(TransferJob::ReportCsv(true)));
    assert!(h.state().1.contains(&to));
    let vault = sample_vault("report-csv-to");
    let mut worker = worker_new(None);
    let open = Effect::OpenVault(vault.path.display().to_string());
    execute(&mut worker, open, NOW_NS);
    worker.folder_picker = Some(chosen_folder);
    let msgs = execute(&mut worker, to.clone(), NOW_NS);
    let Msg::Library(LibraryMsg::Transfer(TransferMsg::Exported(Ok(path)))) = &msgs[0] else {
        panic!("{:?}", msgs[0]);
    };
    assert!(
        path.starts_with(chosen_folder().unwrap().to_str().unwrap()),
        "{path}"
    );
    let text = std::fs::read_to_string(path).unwrap();
    for want in ["summary,all,learned,", "summary,all,streak_days,", "stack,"] {
        assert!(text.contains(want), "missing {want:?} in {text}");
    }
    worker.folder_picker = Some(cancelled);
    let msgs = execute(&mut worker, to, NOW_NS);
    assert!(format!("{:?}", msgs[0]).contains("No folder was chosen"));
}
