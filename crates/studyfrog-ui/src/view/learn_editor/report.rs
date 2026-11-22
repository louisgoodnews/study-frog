//! The custom report of the open learn mode (STORY-0198, opt-in): read only when asked for, and shown with the mode
//! alone. Built-in reports never include these runs.

use eframe::egui::{self, RichText};

use crate::learn_editor::{LearnEditor, LearnMsg};
use crate::msg::Msg;

/// The "Show report" button, and the report once it has been read.
pub fn section(e: &LearnEditor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new("Custom report").strong());
        if ui
            .add_enabled(e.file.is_some(), egui::Button::new("Show report"))
            .on_hover_text("Reads this mode's runs only. Built-in reports never include them; nothing is read until you press this.")
            .clicked()
        {
            out.push(Msg::Learn(LearnMsg::ShowReport));
        }
    });
    let Some(r) = &e.report else { return };
    ui.label(summary_line(r.runs, r.finished, r.items));
    list(ui, "Entries per step", &r.visits);
    list(ui, "Ratings given", &r.ratings);
    list(ui, "Branches taken", &r.branches);
}

/// "N runs (M finished), K items reached." with each noun singular for 1.
fn summary_line(runs: i64, finished: i64, items: i64) -> String {
    format!(
        "{runs} {} ({finished} finished), {items} {} reached.",
        noun(runs, "run", "runs"),
        noun(items, "item", "items"),
    )
}

fn noun<'a>(n: i64, one: &'a str, many: &'a str) -> &'a str {
    if n == 1 { one } else { many }
}

fn list(ui: &mut egui::Ui, title: &str, rows: &[(String, i64)]) {
    if rows.is_empty() {
        return;
    }
    ui.weak(title);
    ui.horizontal_wrapped(|ui| {
        for (name, n) in rows {
            ui.monospace(format!("{name}: {n}"));
            ui.add_space(8.0);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::summary_line;

    #[test]
    fn nouns_agree_with_their_counts() {
        assert_eq!(summary_line(1, 1, 1), "1 run (1 finished), 1 item reached.");
        assert_eq!(
            summary_line(2, 0, 0),
            "2 runs (0 finished), 0 items reached."
        );
    }
}
