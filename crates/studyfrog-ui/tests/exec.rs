#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The executor and update together, against a real vault and a real config directory.

mod common;

use std::fs;

use common::{run_to_idle, sample_vault, temp_dir};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::worker_new;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::{NoticeKind, Screen};

#[test]
fn startup_reopens_the_remembered_vault_and_lands_on_home() {
    let vault = sample_vault("restore");
    let cfg = temp_dir("cfg");
    fs::write(
        cfg.path.join("app.toml"),
        format!(
            "vault = {:?}\nrecent = [{:?}]\n",
            vault.path.display().to_string(),
            vault.path.display().to_string()
        ),
    )
    .unwrap();
    let mut worker = worker_new(Some(cfg.path.clone()));
    let mut model = Model::default();

    run_to_idle(&mut model, &mut worker, vec![Effect::LoadAppSettings]);

    let s = &model.state;
    assert_eq!(s.screen, Screen::Home);
    let v = s.vault.as_ref().unwrap();
    let rows: Vec<_> = v
        .overview
        .stacks
        .iter()
        .map(|r| (r.title.as_str(), r.notes, r.cards, r.questions))
        .collect();
    assert_eq!(rows, [("French", 1, 2, 0)]);
    assert_eq!(v.overview.inbox_items, 1);
    assert!(
        vault.path.join(".studyfrog/index.sqlite").exists(),
        "the index lives inside the vault, hidden"
    );
    assert!(
        fs::read_to_string(cfg.path.join("app.toml"))
            .unwrap()
            .contains("recent"),
        "settings were saved again"
    );
}

#[test]
fn a_first_run_without_settings_stays_on_the_welcome_screen() {
    let cfg = temp_dir("cfg-empty");
    let mut model = Model::default();
    run_to_idle(
        &mut model,
        &mut worker_new(Some(cfg.path.join("none"))),
        vec![Effect::LoadAppSettings],
    );
    assert_eq!(
        (model.state.screen, model.state.vault.is_none()),
        (Screen::Welcome, true)
    );
    assert!(model.state.notice.is_none());
}

#[test]
fn opening_a_missing_folder_reports_it_and_a_real_one_works_afterwards() {
    let vault = sample_vault("open");
    let cfg = temp_dir("cfg-open");
    let mut worker = worker_new(Some(cfg.path.clone()));
    let mut model = Model::default();

    apply(
        &mut model,
        Msg::PathEdited(vault.path.join("nope").display().to_string()),
    );
    let fx = apply(&mut model, Msg::OpenVaultClicked);
    run_to_idle(&mut model, &mut worker, fx);
    assert_eq!(model.state.screen, Screen::Welcome);
    assert_eq!(model.state.notice.as_ref().unwrap().kind, NoticeKind::Error);
    assert!(
        model
            .state
            .notice
            .as_ref()
            .unwrap()
            .text
            .contains("not a folder")
    );

    apply(
        &mut model,
        Msg::PathEdited(vault.path.display().to_string()),
    );
    let fx = apply(&mut model, Msg::OpenVaultClicked);
    run_to_idle(&mut model, &mut worker, fx);
    assert_eq!(model.state.screen, Screen::Home);
    assert_eq!(model.state.app.recent, [vault.path.display().to_string()]);
}

#[test]
fn ticks_pick_up_new_files_and_settings_changes_without_reopening() {
    let vault = sample_vault("poll");
    let mut worker = worker_new(None);
    let mut model = Model::default();
    let fx = apply(
        &mut model,
        Msg::OpenRecent(vault.path.display().to_string()),
    );
    run_to_idle(&mut model, &mut worker, fx);
    let inbox = |m: &Model| m.state.vault.as_ref().unwrap().overview.inbox_items;
    assert_eq!(inbox(&model), 1);

    let fx = apply(&mut model, Msg::Tick);
    run_to_idle(&mut model, &mut worker, fx);
    assert!(
        !model.state.polling,
        "a quiet poll changes nothing and ends"
    );

    fs::write(
        vault.path.join("new.md"),
        "---\nid: 0000000000000000000000000Z\nkind: note\ntitle: New\n---\n",
    )
    .unwrap();
    fs::write(
        vault.path.join("studyfrog.toml"),
        "[srs]\nscheduler = \"sm2\"\n",
    )
    .unwrap();
    let fx = apply(&mut model, Msg::Tick);
    run_to_idle(&mut model, &mut worker, fx);
    let v = model.state.vault.as_ref().unwrap();
    assert_eq!(v.overview.inbox_items, 2);
    assert_eq!(
        v.settings.scheduler,
        studyfrog_core::scheduler::Scheduler::Sm2,
        "settings are re-read on change"
    );
}

#[test]
fn an_invalid_studyfrog_toml_does_not_stop_the_vault_from_opening() {
    let vault = sample_vault("badcfg");
    fs::write(
        vault.path.join("studyfrog.toml"),
        "[srs]\nscheduler = \"nope\"\n",
    )
    .unwrap();
    let mut model = Model::default();
    let fx = apply(
        &mut model,
        Msg::OpenRecent(vault.path.display().to_string()),
    );
    run_to_idle(&mut model, &mut worker_new(None), fx);
    let v = model.state.vault.as_ref().unwrap();
    assert!(
        v.settings_error.as_ref().unwrap().contains("line 2"),
        "{:?}",
        v.settings_error
    );
    assert_eq!(
        v.settings,
        studyfrog_core::settings::vault_settings_default()
    );
}
