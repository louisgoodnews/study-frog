#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! STORY-0122: "Open logs folder" in Settings > Data through an injected opener, and worker events
//! (failures, import and export outcomes) written to the application log.

mod common;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_ui::app_settings_ui::{SettingsTab, SettingsTabMsg};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{execute, worker_new};
use studyfrog_ui::exec_log::execute_logged;
use studyfrog_ui::library_bulk::LibraryEffect;
use studyfrog_ui::logs_ui::LogsMsg;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::notify::NotifyEffect;
use studyfrog_ui::state::{NoticeKind, Screen};
use studyfrog_ui::transfer_types::TransferJob;

static OPENED: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());

fn record(dir: &Path) -> Result<(), String> {
    OPENED.lock().unwrap().push(dir.to_owned());
    Ok(())
}

fn broken(_: &Path) -> Result<(), String> {
    Err("xdg-open could not be started: not found".into())
}

fn scratch(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sf-logs-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn settings_data_has_an_open_logs_folder_button_that_asks_the_worker() {
    let mut m = open_model();
    m.state.screen = Screen::Settings;
    apply(
        &mut m,
        Msg::SettingsTab(SettingsTabMsg::Select(SettingsTab::Data)),
    );
    let mut h = harness(m);
    h.run_steps(3);
    h.get_by_label("Open logs folder").click();
    h.run_steps(2);
    let open = Effect::Notifications(NotifyEffect::OpenLogs);
    assert!(h.state().1.contains(&open), "{:?}", h.state().1);
    assert_all_named(&h, "settings data");
}

#[test]
fn the_worker_opens_the_logs_folder_with_the_hosts_opener_and_says_how_it_went() {
    let logs = scratch("open");
    let mut w = worker_new(None);
    w.state_dir = Some(logs.clone());
    let open = || Effect::Notifications(NotifyEffect::OpenLogs);
    let msgs = execute(&mut w, open(), 1);
    assert!(
        format!("{msgs:?}").contains("cannot start a file manager"),
        "{msgs:?}"
    );
    w.opener = Some(record);
    let msgs = execute(&mut w, open(), 1);
    assert_eq!(
        OPENED.lock().unwrap().as_slice(),
        std::slice::from_ref(&logs)
    );
    assert!(logs.is_dir(), "the folder is made if it is missing");
    let mut m = open_model();
    for msg in msgs {
        apply(&mut m, msg);
    }
    let notice = m.state.notice.clone().unwrap();
    assert_eq!(notice.kind, NoticeKind::Info);
    assert!(notice.text.contains("Opened the logs folder"));
    w.opener = Some(broken);
    let msgs = execute(&mut w, open(), 1);
    let failed = Msg::Logs(LogsMsg::Opened(Err(
        "xdg-open could not be started: not found".into(),
    )));
    assert_eq!(msgs, vec![failed]);
}

#[test]
fn failed_effects_and_export_outcomes_go_to_the_application_log() {
    let logs = scratch("events");
    let mut w = worker_new(None);
    w.state_dir = Some(logs.clone());
    execute_logged(
        &mut w,
        Effect::OpenVault("/no/such/vault/here".into()),
        1_800_000_000_000_000_000,
    );
    let vault = common::sample_vault("logs-export");
    execute_logged(
        &mut w,
        Effect::OpenVault(vault.path.display().to_string()),
        1,
    );
    let csv = Effect::Library(LibraryEffect::Transfer(TransferJob::ReportCsv(false)));
    execute_logged(&mut w, csv, 1_800_000_000_000_000_000);
    let text = std::fs::read_to_string(logs.join("studyfrog.log")).unwrap();
    assert!(text.contains("OpenVault failed:"), "{text}");
    assert!(text.contains("export written to"), "{text}");
    let json = std::fs::read_to_string(logs.join("console.jsonl")).unwrap();
    assert!(json.contains("\"source\":\"worker\""), "{json}");
    assert!(json.contains("\"level\":\"warn\""), "{json}");
}
