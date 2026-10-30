//! The performance explorer: where the user is in the vault, how it went, and what is one level down.

use eframe::egui::{self, RichText, Role, WidgetInfo};
use studyfrog_core::plan::item_kind_name;
use studyfrog_md::rfc3339::format_rfc3339;
use studyfrog_report::explore::{Perf, success_percent};
use studyfrog_service::explore::{Explorer, Period, Row, RowKind, Scope};

use crate::explorer_ui::ExplorerMsg;
use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::{explorer_charts, kit};

const PERIODS: [(Period, &str); 3] = [
    (Period::All, "All time"),
    (Period::Days(30), "Last 30 days"),
    (Period::Days(7), "Last 7 days"),
];

fn send(out: &mut Vec<Msg>, m: ExplorerMsg) {
    out.push(Msg::Explorer(m));
}

pub(super) fn seconds(ms: u32) -> String {
    format!("{:.1} s", f64::from(ms) / 1000.0)
}

pub(super) fn minutes(ms: u64) -> String {
    match ms / 1000 {
        s if s < 90 => format!("{s} s"),
        s => format!("{} min", s / 60),
    }
}

pub(super) fn day_of(ms: Option<i64>) -> String {
    ms.and_then(|t| format_rfc3339(studyfrog_core::time::UnixMs(t)))
        .map_or("-".to_owned(), |t| t[..10].to_owned())
}

pub(super) fn percent(p: &Perf) -> String {
    success_percent(p).map_or("-".to_owned(), |v| format!("{v} %"))
}

/// The screen.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(e) = &state.explorer.data else {
        ui.label("Looking at your history…");
        return;
    };
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            crumbs(e, ui, out);
            periods(state, ui, out);
            ui.add_space(6.0);
            if e.perf.reviews == 0 {
                ui.label("No reviews here yet for this period.");
            }
            explorer_charts::tiles(&e.perf, ui);
            ui.add_space(8.0);
            ui.columns(2, |c| {
                explorer_charts::ratings(&e.perf, &mut c[0]);
                explorer_charts::chart(&e.chart, &mut c[1]);
            });
            item_history(e, ui, out);
            subjects(e, ui);
            rows(e, ui, out);
        });
}

fn crumbs(e: &Explorer, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal_wrapped(|ui| {
        let last = e.crumbs.len().saturating_sub(1);
        for (i, (label, scope)) in e.crumbs.iter().enumerate() {
            if i == last {
                ui.label(RichText::new(label).size(22.0).strong());
            } else {
                if ui.link(label).clicked() {
                    send(out, ExplorerMsg::Open(scope.clone()));
                }
                ui.label("›");
            }
        }
    });
}

fn periods(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal(|ui| {
        for (p, label) in PERIODS {
            let r = ui.selectable_label(state.explorer.period == p, label);
            r.widget_info(|| WidgetInfo::labeled(Role::Button, true, format!("Period: {label}")));
            if r.clicked() {
                send(out, ExplorerMsg::Period(p));
            }
        }
    });
}

fn item_history(e: &Explorer, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Scope::Item(uid) = &e.scope else { return };
    ui.add_space(8.0);
    if kit::primary_button(ui, "Open the item").clicked() {
        out.push(Msg::OpenItem(*uid));
    }
    kit::section(ui, "Every review");
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        if e.history.is_empty() {
            ui.weak("Not reviewed in this period.");
        }
        for h in &e.history {
            let name = ["Again", "Hard", "Good", "Easy"]
                .get(usize::from(h.rating).saturating_sub(1))
                .copied()
                .unwrap_or("?");
            ui.label(format!("{}  {name}", day_of(Some(h.at_ms))));
        }
    });
}

fn subjects(e: &Explorer, ui: &mut egui::Ui) {
    if e.subjects.is_empty() {
        return;
    }
    ui.add_space(8.0);
    kit::section(ui, "By subject");
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        for (name, p) in &e.subjects {
            ui.horizontal(|ui| {
                ui.label(RichText::new(name).strong());
                ui.label(format!(
                    "{} reviews, {} good or easy",
                    p.reviews,
                    percent(p)
                ));
            });
        }
    });
}

fn rows(e: &Explorer, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if e.rows.is_empty() {
        return;
    }
    ui.add_space(8.0);
    kit::section(
        ui,
        if matches!(e.scope, Scope::Vault) {
            "Stacks"
        } else {
            "Inside"
        },
    );
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        for r in &e.rows {
            row(r, ui, out);
        }
    });
}

fn row(r: &Row, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let pal = palette_of(ui);
    ui.horizontal(|ui| {
        let (word, color) = match r.kind {
            RowKind::Stack => ("stack", pal.accent),
            RowKind::Item(k) => (item_kind_name(k), pal.text_weak),
        };
        kit::pill(ui, word, color);
        if ui.link(&r.label).clicked() {
            send(out, ExplorerMsg::Open(r.target.clone()));
        }
        let weak = |t: String| RichText::new(t).small().color(pal.text_weak);
        ui.label(weak(format!("{} reviews", r.perf.reviews)));
        ui.label(weak(format!("{} good or easy", percent(&r.perf))));
        if let Some(ms) = r.perf.median_ms {
            ui.label(weak(format!("median {}", seconds(ms))));
        }
    });
}
