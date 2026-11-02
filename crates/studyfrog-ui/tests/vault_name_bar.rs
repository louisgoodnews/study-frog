#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The open vault's name in the top bar.

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;

#[test]
fn the_vault_name_is_the_last_folder_and_a_long_one_is_cut() {
    use studyfrog_ui::chrome::{MAX_VAULT_NAME, vault_label};
    assert_eq!(vault_label("/home/me/notes"), "notes");
    assert_eq!(vault_label("/home/me/notes/"), "notes", "a trailing slash");
    assert_eq!(
        vault_label("C:\\Users\\me\\Study"),
        "Study",
        "a Windows path"
    );
    assert_eq!(vault_label("notes"), "notes");
    let long = format!("/v/{}", "Psychologische Diagnostik und Evaluation");
    let shown = vault_label(&long);
    assert_eq!(shown.chars().count(), MAX_VAULT_NAME);
    assert!(shown.ends_with('…') && shown.starts_with("Psychologische"));
    assert_eq!(
        vault_label("/v/exactly-twenty-four-char"),
        "exactly-twenty-four-char"
    );
}

#[test]
fn a_long_vault_name_is_cut_in_the_bar() {
    let mut m = open_model();
    m.state.vault.as_mut().unwrap().root = "/v/Psychologische Diagnostik und Evaluation".into();
    let mut h = harness(m);
    h.run();
    assert!(
        h.query_by_label("Psychologische Diagnostik und Evaluation")
            .is_none()
    );
    assert!(h.query_by_label("Psychologische Diagnost…").is_some());
    assert_all_named(&h, "top bar with a long vault name");
}
