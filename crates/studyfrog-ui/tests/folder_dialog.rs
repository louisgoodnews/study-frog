#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Choosing the vault folder with the native dialog (a fake picker stands in for it).

mod common;

use std::path::PathBuf;

use common::NOW_NS;
use common::views::{assert_all_named, harness};
use egui_kittest::kittest::Queryable;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{execute, worker_new, worker_with_host};
use studyfrog_ui::host::Host;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;

fn nothing() -> Option<PathBuf> {
    None
}

#[test]
fn the_choose_folder_button_asks_for_the_dialog_and_a_chosen_folder_is_opened() {
    let mut h = harness(Model::default());
    h.run();
    assert_all_named(&h, "welcome");
    h.get_by_label("Choose folder…").click();
    h.run();
    assert!(h.state().1.contains(&Effect::PickVaultFolder));

    let mut m = Model::default();
    let fx = apply(&mut m, Msg::VaultFolderPicked(Some("/v/notes".into())));
    assert_eq!(m.state.path_input, "/v/notes");
    assert_eq!(fx, [Effect::OpenVault("/v/notes".into())]);

    let mut m = Model::default();
    assert!(
        apply(&mut m, Msg::VaultFolderPicked(None)).is_empty(),
        "cancelling does nothing"
    );
    assert!(m.state.path_input.is_empty());
}

#[test]
fn the_executor_uses_the_hosts_dialog_and_copes_without_one() {
    fn pick() -> Option<PathBuf> {
        Some(PathBuf::from("/chosen/folder"))
    }
    let mut with = worker_with_host(
        None,
        None,
        Host {
            folder_picker: Some(pick),
            ..Host::default()
        },
    );
    assert_eq!(
        execute(&mut with, Effect::PickVaultFolder, NOW_NS),
        [Msg::VaultFolderPicked(Some("/chosen/folder".into()))]
    );
    let mut cancelled = worker_with_host(
        None,
        None,
        Host {
            folder_picker: Some(nothing),
            ..Host::default()
        },
    );
    assert_eq!(
        execute(&mut cancelled, Effect::PickVaultFolder, NOW_NS),
        [Msg::VaultFolderPicked(None)]
    );
    let mut without = worker_new(None);
    assert_eq!(
        execute(&mut without, Effect::PickVaultFolder, NOW_NS),
        [Msg::VaultFolderPicked(None)]
    );
}
