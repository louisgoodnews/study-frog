//! Overhauled vault switcher menu (STORY-0138): active vault card, recent vaults, and clear actions.

use eframe::egui::{self, RichText};

use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::kit;
use crate::view::popup::close;

pub(crate) fn segments(path: &str) -> Vec<&str> {
    path.split(['/', '\\']).filter(|s| !s.is_empty()).collect()
}

/// Short, human-readable label for a vault path; appends a disambiguating ancestor when
/// another recent vault has the same folder name.
pub fn vault_label(path: &str, recent: &[String]) -> String {
    let segs = segments(path);
    let Some(&name) = segs.last() else {
        return path.to_owned();
    };
    let clashes: Vec<Vec<&str>> = recent
        .iter()
        .filter(|p| p.as_str() != path)
        .map(|p| segments(p))
        .filter(|s| s.last() == Some(&name))
        .collect();
    if clashes.is_empty() {
        return name.to_owned();
    }
    for &anc in segs.iter().rev().skip(1) {
        if clashes
            .iter()
            .any(|c| !c.iter().rev().skip(1).any(|&x| x == anc))
        {
            return format!("{name} · {anc}");
        }
    }
    segs.iter()
        .rev()
        .nth(1)
        .map_or_else(|| name.to_owned(), |p| format!("{name} · {p}"))
}

/// Draw the overhauled vault switcher menu.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let p = palette_of(ui);
    ui.horizontal(|ui| {
        ui.strong("Vaults");
        let count = state.app.recent.len();
        if count > 0 {
            kit::pill(ui, &format!("{count} recent"), p.text_weak);
        }
    });
    ui.add_space(6.0);

    if let Some(v) = &state.vault {
        kit::card(ui, |ui| {
            ui.horizontal(|ui| {
                kit::pill(ui, "Active", p.accent);
                if let Some(&parent) = segments(&v.root).iter().rev().nth(1) {
                    ui.weak(parent);
                }
            });
            ui.add_space(2.0);
            let width = ui.available_width();
            let active = egui::Button::selectable(true, &v.name).min_size(egui::vec2(width, 28.0));
            ui.add(active).on_hover_text(&v.root);
        });
        ui.add_space(6.0);
    }

    let others: Vec<&String> = state
        .app
        .recent
        .iter()
        .filter(|p| Some(p.as_str()) != state.vault.as_ref().map(|v| v.root.as_str()))
        .collect();

    if !others.is_empty() {
        ui.weak("Recent vaults");
        ui.add_space(2.0);
        for path in others {
            let label = vault_label(path, &state.app.recent);
            let width = ui.available_width();
            let row = egui::Button::selectable(false, &label).min_size(egui::vec2(width, 28.0));
            if ui.add(row).on_hover_text(path.as_str()).clicked() {
                out.push(Msg::OpenRecent(path.clone()));
                close(out);
            }
        }
        ui.add_space(4.0);
    }

    ui.separator();
    ui.add_space(4.0);

    let width = ui.available_width();
    let open_other =
        egui::Button::selectable(false, "Open another vault…").min_size(egui::vec2(width, 28.0));
    if ui.add(open_other).clicked() {
        out.push(Msg::CloseVault);
        close(out);
    }

    if ui
        .add(
            egui::Button::new(RichText::new("Choose folder…").color(p.accent))
                .min_size(egui::vec2(width, 28.0)),
        )
        .clicked()
    {
        out.push(Msg::BrowseVaultClicked);
        close(out);
    }
}
