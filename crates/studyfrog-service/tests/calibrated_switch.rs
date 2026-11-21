#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The switch written from the app keeps the rest of `studyfrog.toml` and reads back (STORY-0177).

mod fam_support;

use fam_support::vault;
use studyfrog_config::vault::VAULT_CONFIG_FILE;
use studyfrog_service::session::open_vault;
use studyfrog_service::srs_setting::apply_calibrated_start;
use studyfrog_store::vault_config::read_vault_settings;

#[test]
fn the_switch_is_written_kept_in_the_file_and_read_back() {
    let dir = vault("calibrated-switch", None);
    let path = dir.0.join(VAULT_CONFIG_FILE);
    std::fs::write(
        &path,
        "# my vault\n[srs]\nscheduler = \"fsrs\"\nrequest_retention = 0.85\n",
    )
    .unwrap();
    let (mut session, _) = open_vault(dir.0.to_str().unwrap()).unwrap();

    let info = apply_calibrated_start(&mut session, false).expect("written");
    assert!(!info.settings.calibrated_start);
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(
        text.contains("# my vault") && text.contains("request_retention = 0.85"),
        "{text}"
    );
    assert!(!read_vault_settings(&dir.0).unwrap().calibrated_start);

    let info = apply_calibrated_start(&mut session, true).expect("written");
    assert!(info.settings.calibrated_start);
    assert!(read_vault_settings(&dir.0).unwrap().calibrated_start);
}
