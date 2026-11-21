#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The calibrated-start switch in `studyfrog.toml` (STORY-0177): parsed, rendered, and edited in place.

use studyfrog_config::retention_edit::with_setting_line;
use studyfrog_config::vault::{parse_vault_settings, render_vault_settings};
use studyfrog_core::settings::vault_settings_default;

#[test]
fn the_switch_is_on_by_default_and_round_trips() {
    assert!(vault_settings_default().calibrated_start);
    let rendered = render_vault_settings(&vault_settings_default());
    assert!(rendered.contains("calibrated_start = true"), "{rendered}");
    let off = parse_vault_settings("[srs]\ncalibrated_start = false\n").expect("parses");
    assert!(!off.calibrated_start);
    let back = parse_vault_settings(&render_vault_settings(&off)).expect("parses");
    assert!(!back.calibrated_start);
}

#[test]
fn an_absent_switch_is_on() {
    let s = parse_vault_settings("[srs]\nrequest_retention = 0.9\n").expect("parses");
    assert!(s.calibrated_start);
}

#[test]
fn the_line_is_replaced_in_place_and_added_when_absent() {
    let text =
        "# keep me\n[srs]\ncalibrated_start = true\nrequest_retention = 0.9\n\n[other]\nx = 1\n";
    let off = with_setting_line(
        text,
        "calibrated_start",
        "calibrated_start = false".to_owned(),
        Some("srs"),
    );
    assert!(off.contains("calibrated_start = false"));
    assert!(!off.contains("calibrated_start = true"));
    assert!(off.contains("# keep me") && off.contains("[other]") && off.contains("x = 1"));
    let added = with_setting_line(
        "[srs]\nrequest_retention = 0.9\n",
        "calibrated_start",
        "calibrated_start = false".to_owned(),
        Some("srs"),
    );
    assert!(
        added.contains("calibrated_start = false") && added.contains("request_retention = 0.9")
    );
}
