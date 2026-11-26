//! Nautilus-style bottom selection status pill for Folders (EPIC-074).

use eframe::egui::{self, CornerRadius, RichText, Stroke};

use crate::folders_browse::{child_count, display_name, visible_dir_entries};
use crate::folders_tree::entries_from_vault;
use crate::folders_types::FolderEntry;
use crate::state::State;
use crate::theme::palette_of;

/// Human-readable file size (`"512 B"`, `"14.2 KB"`, `"2.4 MB"`).
pub(super) fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

/// Human-readable modified label relative to `now_ms`.
pub(super) fn format_modified(mtime_ns: i64, now_ms: i64) -> String {
    if mtime_ns <= 0 {
        return "-".to_owned();
    }
    let mtime_ms = mtime_ns / 1_000_000;
    if now_ms <= 0 {
        return "Recorded".to_owned();
    }
    match (now_ms - mtime_ms).div_euclid(86_400_000) {
        i64::MIN..=0 => "Today".to_owned(),
        1 => "Yesterday".to_owned(),
        n => format!("{n} days ago"),
    }
}

fn effective_entries(state: &State) -> Vec<FolderEntry> {
    if state.folders.entries.is_empty()
        && let Some(v) = &state.vault
    {
        return entries_from_vault(v, &state.library.items);
    }
    state.folders.entries.clone()
}

/// Draw the compact bottom-right status pill describing the current selection or folder count.
pub fn draw(state: &State, ui: &mut egui::Ui) {
    let all = effective_entries(state);
    let pal = palette_of(ui);

    let text = if let Some(sel) = &state.folders.selected
        && let Some(entry) = all.iter().find(|e| &e.rel_path == sel)
    {
        let name = display_name(entry);
        let detail = if entry.is_folder {
            let n = child_count(&all, &entry.rel_path);
            if n == 1 {
                "1 item".to_owned()
            } else {
                format!("{n} items")
            }
        } else {
            format_size(entry.size_bytes)
        };
        format!("\"{name}\" selected ({detail})")
    } else {
        let count = visible_dir_entries(&all, &state.folders).len();
        if count == 1 {
            "1 item in view".to_owned()
        } else {
            format!("{count} items in view")
        }
    };

    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        egui::Frame::new()
            .fill(pal.surface)
            .stroke(Stroke::new(1.0, pal.border))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(egui::Margin::symmetric(10, 4))
            .show(ui, |ui| {
                ui.label(RichText::new(text).small().color(pal.text_weak));
            });
    });
}
