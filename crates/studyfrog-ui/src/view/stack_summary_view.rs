//! The Summary tab of a stack: four cards, the status ring, priorities, types, the activity feed and the sub-stacks.
//! Every chart carries its numbers as text, so nothing depends on seeing a colour.

use std::f32::consts::TAU;

use eframe::egui::{self, ProgressBar, Stroke, vec2};
use studyfrog_core::priority::{PRIORITIES, priority_name};
use studyfrog_core::status::{STATUSES, status_label};
use studyfrog_service::stack_summary::{Activity, Happened, StackSummary, SubStack, WINDOW_DAYS};

use crate::msg::Msg;
use crate::state::{Selection, State};
use crate::theme::palette_of;
use crate::view::kit;

fn cards(s: &StackSummary, ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        for (n, caption) in [
            (
                s.completed,
                format!("completed in the last {WINDOW_DAYS} days"),
            ),
            (s.updated, format!("updated in the last {WINDOW_DAYS} days")),
            (s.created, format!("created in the last {WINDOW_DAYS} days")),
            (s.due_soon, format!("due in the next {WINDOW_DAYS} days")),
        ] {
            kit::card(ui, |ui| kit::stat(ui, &n.to_string(), &caption));
        }
    });
}

/// A ring of one arc per status; the legend beside it says each name, count and share.
fn ring(s: &StackSummary, ui: &mut egui::Ui) {
    let total: usize = s.statuses.iter().sum();
    let pal = palette_of(ui);
    let colors = [
        pal.text_weak,
        pal.accent,
        pal.success,
        pal.border,
        pal.warning,
    ];
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(vec2(120.0, 120.0), egui::Sense::hover());
        let (center, radius) = (rect.center(), 50.0);
        let mut from = -TAU / 4.0;
        for (i, n) in s.statuses.iter().enumerate().filter(|(_, n)| **n > 0) {
            let sweep = TAU * *n as f32 / total.max(1) as f32;
            let steps = ((sweep / TAU) * 64.0).ceil().max(2.0) as usize;
            let points: Vec<egui::Pos2> = (0..=steps)
                .map(|k| {
                    let a = from + sweep * k as f32 / steps as f32;
                    center + vec2(a.cos(), a.sin()) * radius
                })
                .collect();
            ui.painter()
                .add(egui::Shape::line(points, Stroke::new(14.0, colors[i])));
            from += sweep;
        }
        ui.painter().text(
            center,
            egui::Align2::CENTER_CENTER,
            total.to_string(),
            egui::FontId::proportional(22.0),
            pal.text,
        );
        ui.vertical(|ui| {
            ui.label(format!("{total} item(s) in all"));
            for (st, n) in STATUSES.iter().zip(s.statuses) {
                let share = (n * 100).checked_div(total).unwrap_or(0);
                ui.label(format!("{}: {n} ({share} %)", status_label(*st)));
            }
        });
    });
}

fn priorities(s: &StackSummary, ui: &mut egui::Ui) {
    let top = s.priorities.iter().copied().max().unwrap_or(0).max(1) as f32;
    for (p, n) in PRIORITIES.iter().zip(s.priorities) {
        ui.horizontal(|ui| {
            ui.label(priority_name(*p));
            ui.add(
                ProgressBar::new(n as f32 / top)
                    .desired_width(220.0)
                    .text(n.to_string()),
            );
        });
    }
}

fn types(s: &StackSummary, ui: &mut egui::Ui) {
    egui::Grid::new("stack-types")
        .num_columns(2)
        .show(ui, |ui| {
            for (name, n) in [
                ("Notes", s.types[0]),
                ("Cards", s.types[1]),
                ("Questions", s.types[2]),
            ] {
                ui.label(name);
                ui.label(n.to_string());
                ui.end_row();
            }
        });
}

fn what(a: &Activity) -> &'static str {
    match a.what {
        Happened::Created => "created",
        Happened::Edited => "edited",
        Happened::Rehearsed => "rehearsed",
    }
}

fn feed(s: &StackSummary, now_ms: i64, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if s.activity.is_empty() {
        ui.weak("No activity yet");
    }
    for a in &s.activity {
        ui.horizontal_wrapped(|ui| {
            if ui.link(&a.title).clicked() {
                out.push(Msg::OpenItem(a.uid));
            }
            ui.weak(format!("{} {}", what(a), super::home::ago(now_ms, a.at_ms)));
        });
    }
}

fn substack(sub: &SubStack, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal_wrapped(|ui| {
        if ui.link(&sub.title).clicked() {
            out.push(Msg::Select(Selection::Stack(sub.folder.clone())));
        }
        let familiarity = sub
            .percent
            .map_or("no history".to_owned(), |p| format!("familiarity {p} %"));
        ui.weak(format!(
            "{} - {} of {} done - {familiarity}",
            status_label(sub.status),
            sub.done,
            sub.total
        ));
    });
}

/// The tab.
pub fn draw(state: &State, folder: &str, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let held = state
        .library
        .stack_view
        .summary
        .as_ref()
        .filter(|(f, _, _)| f == folder);
    let Some((_, _, s)) = held else {
        ui.label("Reading the summary…");
        return;
    };
    let now_ms = state.wall_ms;
    cards(s, ui);
    ui.add_space(8.0);
    kit::section(ui, "Status overview");
    ring(s, ui);
    ui.add_space(8.0);
    kit::section(ui, "Priorities");
    priorities(s, ui);
    ui.add_space(8.0);
    kit::section(ui, "Types");
    types(s, ui);
    ui.add_space(8.0);
    kit::section(ui, "Activity");
    feed(s, now_ms, ui, out);
    ui.add_space(8.0);
    kit::section(ui, "Sub-stacks");
    if s.substacks.is_empty() {
        ui.weak("No stacks inside this one.");
    }
    for sub in &s.substacks {
        substack(sub, ui, out);
    }
}
