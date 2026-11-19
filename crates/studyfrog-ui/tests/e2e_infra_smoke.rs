#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Smoke tests verifying E1 shared E2E helpers, `Host` picker stubs, input/hover, and all fixture builders.

mod common;

use std::fs;

use common::NOW_NS;
use common::TempDir;
use common::e2e_helpers::{
    advance_ticks, boot_sample_harness, button_rects_in_band, clear_pickers, click_label,
    count_buttons_in_band, find_by_label_contains, hover_for_tooltip, press_key, press_shortcut,
    rect_inside_bounds, rects_overlap, set_picked_file, set_picked_folder, stub_file_picker,
    stub_folder_picker,
};
use common::e2e_helpers_fixtures::{
    write_apkg_fixture, write_bad_pdf, write_corrupt_zip, write_csv_fixture, write_empty_file,
    write_export_zip_fixture, write_invalid_utf8_file, write_json_fixture, write_md_card_fixture,
    write_md_note_fixture, write_obsidian_vault_fixture, write_obsidian_zip_fixture,
    write_pdf_fixture, write_ron_fixture, write_toml_fixture, write_tsv_fixture,
};
use common::e2e_helpers_hw::{
    clear_mock_notifications_for, homework_toml_exists, primary_button_count,
    record_mock_notification, rhythm_snapshot, take_mock_notifications_for, vault_item_counts,
};
use common::temp_dir;
use common::views::App;
use eframe::egui::accesskit::Role;
use eframe::egui::{Key, Modifiers, Pos2, Rect, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use studyfrog_service::pdf_library::add_pdf;
use studyfrog_service::pdfs::{index_pdfs_fully, pdf_library};
use studyfrog_service::session::open_vault;
use studyfrog_service::xfer_adapters::read_source;
use studyfrog_ui::exec::Worker;
use studyfrog_ui::state::Screen;

#[test]
fn e2e_harness_boots_steps_ticks_keys_hover_and_accesskit_queries() {
    let (dir, mut worker, mut h) = boot_sample_harness("smoke-harness");
    assert_boot_state(&h, &dir);
    assert_picker_stubs(&dir);
    assert_ticks_and_tooltip(&mut h, &mut worker);
    assert_top_band_layout(&h);
    assert_click_and_keys(&mut h, &mut worker);
}

fn assert_boot_state(h: &Harness<'_, App>, dir: &TempDir) {
    let counts = vault_item_counts(&h.state().0);
    assert_eq!(counts, (1, 1, 2, 1));
    assert!(!homework_toml_exists(&dir.path));
    assert!(primary_button_count(h) <= 1);
    assert!(rhythm_snapshot(h).section > rhythm_snapshot(h).card);
}

fn assert_picker_stubs(dir: &TempDir) {
    let file = dir.path.join("sample.csv");
    set_picked_file(Some(file.clone()));
    set_picked_folder(Some(dir.path.clone()));
    assert_eq!(stub_file_picker(&["csv"]), Some(file));
    assert_eq!(stub_folder_picker(), Some(dir.path.clone()));
    clear_pickers();
    assert_eq!(stub_file_picker(&["csv"]), None);
}

fn assert_ticks_and_tooltip(h: &mut Harness<'_, App>, worker: &mut Worker) {
    let before_ticks = h.state().0.state.reminder_ticks;
    advance_ticks(h, worker, 2);
    assert!(h.state().0.state.reminder_ticks >= before_ticks + 2);

    hover_for_tooltip(h, worker, "Application settings");
    assert!(find_by_label_contains(h, "Application settings").is_some());
    assert_eq!(h.query_all_by_role(Role::Tooltip).count(), 1);
}

fn assert_top_band_layout(h: &Harness<'_, App>) {
    let top_buttons = button_rects_in_band(h, 0.0, 48.0);
    assert!(count_buttons_in_band(h, 0.0, 48.0) >= 3);
    let viewport = Rect::from_min_size(Pos2::ZERO, vec2(1280.0, 800.0));
    for (_, rect) in &top_buttons {
        assert!(rect_inside_bounds(*rect, viewport));
    }
    assert!(!rects_overlap(
        Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(10.0, 10.0)),
        Rect::from_min_max(Pos2::new(10.0, 0.0), Pos2::new(20.0, 10.0)),
    ));
}

fn assert_click_and_keys(h: &mut Harness<'_, App>, worker: &mut Worker) {
    click_label(h, worker, "Application settings");
    assert_eq!(h.state().0.state.screen, Screen::Settings);
    press_key(h, worker, Key::F1);
    assert_eq!(h.state().0.state.screen, Screen::Help);
    press_shortcut(h, worker, Modifiers::ALT, Key::ArrowLeft);
    assert_eq!(h.state().0.state.screen, Screen::Settings);
}

#[test]
fn e2e_fixtures_parse_cleanly_through_service_adapters_and_pdf_indexer() {
    let dir = temp_dir("smoke-fixtures");
    let csv_h = write_csv_fixture(&dir.path, "headered.csv", true);
    let csv_n = write_csv_fixture(&dir.path, "headless.csv", false);
    let tsv_h = write_tsv_fixture(&dir.path, "headered.tsv", true);
    let tsv_n = write_tsv_fixture(&dir.path, "headless.tsv", false);
    let json = write_json_fixture(&dir.path, "export.json");
    let toml = write_toml_fixture(&dir.path, "export.toml");
    let ron = write_ron_fixture(&dir.path, "export.ron");
    let exp_zip = write_export_zip_fixture(&dir.path, "export.zip");
    let obs_zip = write_obsidian_zip_fixture(&dir.path, "obsidian.zip");
    let obs_dir = write_obsidian_vault_fixture(&dir.path, "obsidian-vault");
    let apkg = write_apkg_fixture(&dir.path, "deck.apkg");

    for path in [
        csv_h, csv_n, tsv_h, tsv_n, json, toml, ron, exp_zip, obs_zip, obs_dir, apkg,
    ] {
        let (bundle, adapter) = read_source(&path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
        assert!(!bundle.items.is_empty(), "{adapter} returned empty items");
    }

    let pdf_src = write_pdf_fixture(&dir.path, "lecture.pdf", "Physics 101", &["Maxwell laws"]);
    let vault = common::sample_vault("smoke-pdf-vault");
    let (mut session, _) = open_vault(vault.path.to_str().unwrap()).unwrap();
    let rel = add_pdf(&session, &pdf_src, "french").unwrap();
    assert_eq!(rel, "french/lecture.pdf");
    index_pdfs_fully(&mut session, NOW_NS).unwrap();
    let rows = pdf_library(&session).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].title, "Physics 101");
    assert_eq!(rows[0].pages, 1);
    assert_eq!(rows[0].error, None);
}

#[test]
fn e2e_md_negative_fixtures_and_mock_notifier_behave_deterministically() {
    let dir = temp_dir("smoke-neg");
    let md_note = write_md_note_fixture(&dir.path, "note.md");
    let md_card = write_md_card_fixture(&dir.path, "card.md");
    assert!(fs::read_to_string(md_note).unwrap().contains("#membrane"));
    assert!(fs::read_to_string(md_card).unwrap().contains("???"));

    let empty = write_empty_file(&dir.path, "empty.csv");
    let bad_zip = write_corrupt_zip(&dir.path, "bad.zip");
    let bad_pdf = write_bad_pdf(&dir.path, "bad.pdf");
    let bad_utf8 = write_invalid_utf8_file(&dir.path, "bad.json");
    assert_eq!(fs::metadata(&empty).unwrap().len(), 0);
    assert!(read_source(&bad_zip).is_err());
    assert!(read_source(&bad_utf8).is_err());
    assert!(fs::read(&bad_pdf).unwrap().starts_with(b"%PDF-1.4"));

    let tag = "[smoke-tag-42]";
    clear_mock_notifications_for(tag);
    record_mock_notification("[other-tag] Ignored", "unrelated").unwrap();
    record_mock_notification("[smoke-tag-42] HW Due", "Problem Set 1").unwrap();
    let mine = take_mock_notifications_for(tag);
    assert_eq!(mine.len(), 1);
    assert_eq!(mine[0].1, "Problem Set 1");
    assert_eq!(take_mock_notifications_for("[other-tag]").len(), 1);
}
