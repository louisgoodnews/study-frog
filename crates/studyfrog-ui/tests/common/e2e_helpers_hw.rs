#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)]
//! Shared opaque-box E2E helpers for UX spacing, primary counts, Homework persistence, and notifications.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use egui_kittest::Harness;
use studyfrog_ui::exec::Worker;
use studyfrog_ui::model::Model;
use studyfrog_ui::theme_space::{Rhythm, primaries, rhythm};

use super::TempDir;
use super::e2e_helpers::boot_sample_harness;
use super::views::App;

static MOCK_NOTIFICATIONS: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());

/// Count how many accent-filled `kit::primary_button` controls rendered on the latest pass.
pub fn primary_button_count(h: &Harness<'_, App>) -> usize {
    primaries(&h.ctx).1
}

/// Snapshot the active `Rhythm` tokens derived from the current `Appearance::density`.
pub fn rhythm_snapshot(h: &Harness<'_, App>) -> Rhythm {
    rhythm(h.state().0.state.app.appearance.density)
}

/// Path to `.studyfrog/homework.toml` inside `vault_root`.
pub fn homework_toml_path(vault_root: &Path) -> PathBuf {
    vault_root.join(".studyfrog").join("homework.toml")
}

/// Return `true` if `.studyfrog/homework.toml` exists on disk as a regular file.
pub fn homework_toml_exists(vault_root: &Path) -> bool {
    homework_toml_path(vault_root).is_file()
}

/// Read raw `.studyfrog/homework.toml` contents from disk.
pub fn read_homework_toml(vault_root: &Path) -> Result<String, std::io::Error> {
    fs::read_to_string(homework_toml_path(vault_root))
}

/// Boot an isolated `sample_vault` harness with `Host` picker stubs ready for UI flows.
pub fn open_vault_harness(label: &str) -> (TempDir, Worker, Harness<'static, App>) {
    boot_sample_harness(label)
}

/// Summarize `(stacks, notes, cards_and_questions, inbox_items)` from `VaultOverview`.
pub fn vault_item_counts(model: &Model) -> (usize, usize, usize, usize) {
    let overview = &model
        .state
        .vault
        .as_ref()
        .expect("vault must be open")
        .overview;
    let notes: usize = overview.stacks.iter().map(|s| s.notes).sum();
    let cards: usize = overview.stacks.iter().map(|s| s.cards + s.questions).sum();
    (overview.stacks.len(), notes, cards, overview.inbox_items)
}

/// Non-capturing `Host` notifier hook recording `(title, body)` for parallel E4 tests.
pub fn record_mock_notification(title: &str, body: &str) -> Result<(), String> {
    let mut log = MOCK_NOTIFICATIONS
        .lock()
        .map_err(|_| "mock notification lock poisoned".to_owned())?;
    log.push((title.to_owned(), body.to_owned()));
    Ok(())
}

/// Drain and return only mock notifications whose title or body contains `tag`.
pub fn take_mock_notifications_for(tag: &str) -> Vec<(String, String)> {
    let mut log = MOCK_NOTIFICATIONS.lock().unwrap();
    let mut matched = Vec::new();
    let mut remaining = Vec::new();
    for entry in log.drain(..) {
        if entry.0.contains(tag) || entry.1.contains(tag) {
            matched.push(entry);
        } else {
            remaining.push(entry);
        }
    }
    *log = remaining;
    matched
}

/// Clear any stale mock notifications matching `tag` before starting a test.
pub fn clear_mock_notifications_for(tag: &str) {
    let _ = take_mock_notifications_for(tag);
}
