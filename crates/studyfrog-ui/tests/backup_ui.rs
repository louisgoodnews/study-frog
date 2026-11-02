#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The backup section of the settings screen, and the worker doing a real backup and restore.

mod common;

use std::path::PathBuf;

use common::views::{assert_all_named, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_config::backup::Manifest;
use studyfrog_ui::backup_ui::{BackupMsg, Chosen};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{execute, worker_new};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

fn tall(m: Model) -> egui_kittest::Harness<'static, common::views::App> {
    egui_kittest::Harness::builder()
        .with_size(eframe::egui::vec2(1000.0, 1600.0))
        .build_ui_state(
            |ui, (model, effects): &mut common::views::App| {
                effects.extend(studyfrog_ui::model::frame(model, ui))
            },
            (m, Vec::new()),
        )
}

fn manifest() -> Manifest {
    Manifest {
        format: 1,
        created_ms: 1_800_000_000_000,
        app_version: "0.1.0".into(),
        vault_name: "notes".into(),
        files: 12,
        bytes: 4096,
        has_settings: true,
        has_keys: false,
    }
}

fn model() -> Model {
    let mut m = open_model();
    m.state.screen = Screen::Settings;
    m.state.app.settings_tab = Some("data".into());
    m
}

#[test]
fn the_buttons_ask_for_a_backup_and_a_chosen_backup_can_be_restored_or_dropped() {
    let mut m = model();
    assert!(
        apply(&mut m, Msg::Backup(BackupMsg::Restore)).is_empty(),
        "nothing chosen yet"
    );
    apply(&mut m, Msg::Backup(BackupMsg::IncludeKeys(true)));
    let fx = apply(&mut m, Msg::Backup(BackupMsg::Start));
    assert_eq!(fx, [Effect::Backup { include_keys: true }]);
    assert!(
        apply(&mut m, Msg::Backup(BackupMsg::Start)).is_empty(),
        "one at a time"
    );
    apply(
        &mut m,
        Msg::Backup(BackupMsg::Done(Ok(Some("Saved x.tar".into())))),
    );
    assert!(!m.state.backup.busy && m.state.backup.last.as_deref() == Some("Saved x.tar"));
    apply(&mut m, Msg::Backup(BackupMsg::Done(Ok(None))));
    assert_eq!(
        m.state.backup.last.as_deref(),
        Some("Saved x.tar"),
        "a cancelled dialog changes nothing"
    );

    assert_eq!(
        apply(&mut m, Msg::Backup(BackupMsg::Choose)),
        [Effect::ChooseBackup]
    );
    apply(
        &mut m,
        Msg::Backup(BackupMsg::Chosen(Ok(Some(Chosen {
            path: "/b/x.tar".into(),
            manifest: manifest(),
        })))),
    );
    apply(&mut m, Msg::Backup(BackupMsg::RestoreSettings(true)));
    let fx = apply(&mut m, Msg::Backup(BackupMsg::Restore));
    assert_eq!(
        fx,
        [Effect::Restore {
            archive: "/b/x.tar".into(),
            with_settings: true
        }]
    );
    let fx = apply(
        &mut m,
        Msg::Backup(BackupMsg::Restored(Ok(Some("/new/vault".into())))),
    );
    assert_eq!(
        fx,
        [Effect::OpenVault("/new/vault".into())],
        "the restored vault is opened"
    );
    assert!(m.state.backup.chosen.is_none());
    apply(
        &mut m,
        Msg::Backup(BackupMsg::Restored(Err("disk full".into()))),
    );
    assert!(m.state.notice.as_ref().unwrap().text.contains("disk full") && !m.state.backup.busy);
}

#[test]
fn the_section_shows_the_options_and_the_chosen_backup() {
    let mut m = model();
    apply(
        &mut m,
        Msg::Backup(BackupMsg::Chosen(Ok(Some(Chosen {
            path: "/b/x.tar".into(),
            manifest: manifest(),
        })))),
    );
    let mut h = tall(m);
    h.run_steps(2);
    for text in [
        "Backup and restore",
        "Back up now…",
        "Restore from a backup…",
        "Backup of notes from 2027-01-15",
        "Restore into a new folder…",
        "Cancel restore",
        "Also replace my settings with the ones in the backup",
    ] {
        assert!(h.query_all_by_label(text).count() >= 1, "missing {text}");
    }
    h.get_by_label("Cancel restore").click();
    h.run_steps(2);
    assert!(h.state().0.state.backup.chosen.is_none());
    assert_all_named(&h, "backup section");
}

fn pick_folder() -> Option<PathBuf> {
    Some(std::env::temp_dir().join("studyfrog-ui-backup-dest"))
}

fn pick_file(_: &[&str]) -> Option<PathBuf> {
    std::fs::read_dir(std::env::temp_dir().join("studyfrog-ui-backup-dest"))
        .ok()?
        .flatten()
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|e| e == "tar"))
}

#[test]
fn the_worker_backs_up_a_real_vault_and_restores_it_elsewhere() {
    let base = std::env::temp_dir();
    let (vault, dest) = (
        base.join("studyfrog-ui-backup-vault"),
        base.join("studyfrog-ui-backup-dest"),
    );
    let _ = std::fs::remove_dir_all(&vault);
    let _ = std::fs::remove_dir_all(&dest);
    std::fs::create_dir_all(&vault).unwrap();
    std::fs::write(
        vault.join("loose.md"),
        "---\nid: 0000000000000000000000000A\nkind: note\ntitle: Loose\n---\nhi\n",
    )
    .unwrap();
    let mut w = worker_new(None);
    w.folder_picker = Some(pick_folder);
    w.file_picker = Some(pick_file);
    let now = 1_800_000_000_000_000_000;
    execute(&mut w, Effect::OpenVault(vault.display().to_string()), now);
    let done = execute(
        &mut w,
        Effect::Backup {
            include_keys: false,
        },
        now,
    );
    let Msg::Backup(BackupMsg::Done(Ok(Some(text)))) = &done[0] else {
        panic!("{:?}", done[0])
    };
    assert!(text.contains("1 files"), "{text}");
    let chosen = execute(&mut w, Effect::ChooseBackup, now);
    let Msg::Backup(BackupMsg::Chosen(Ok(Some(c)))) = &chosen[0] else {
        panic!("{:?}", chosen[0])
    };
    assert_eq!(
        (c.manifest.files, c.manifest.vault_name.as_str()),
        (1, "studyfrog-ui-backup-vault")
    );
    let _ = std::fs::remove_dir_all(&vault);
    let _ = std::fs::remove_dir_all(&dest);
}

#[test]
fn a_fresh_install_can_restore_a_backup_from_the_welcome_page() {
    let mut h = tall(Model::default());
    h.run_steps(2);
    assert!(h.query_by_label("Restore from a backup…").is_some());
    h.get_by_label("Restore from a backup…").click();
    h.run_steps(2);
    assert!(h.state().1.contains(&Effect::ChooseBackup));
    let mut m = Model::default();
    apply(
        &mut m,
        Msg::Backup(BackupMsg::Chosen(Ok(Some(Chosen {
            path: "/b/x.tar".into(),
            manifest: manifest(),
        })))),
    );
    let mut h = tall(m);
    h.run_steps(2);
    assert!(h.query_by_label("Restore into a new folder…").is_some());
}
