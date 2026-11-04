//! The Self-assessment and Calibration report: compares subjective diary reflections
//! against objective review accuracy and familiarity over time.

use eframe::egui::{self, ProgressBar};
use studyfrog_core::civil::format_date;
use studyfrog_report::calibration::{
    CalibrationBucket, CalibrationReport, bias_explanation, bias_label,
};

use crate::msg::Msg;
use crate::state::State;

fn bar(ui: &mut egui::Ui, label: &str, fraction: f32, text: String) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(
            ProgressBar::new(fraction.clamp(0.0, 1.0))
                .desired_width(160.0)
                .text(text),
        );
    });
}

/// Draw the self-assessment & calibration report tab.
pub fn draw(state: &State, ui: &mut egui::Ui, _out: &mut Vec<Msg>) {
    let Some(reports) = &state.reports else {
        ui.label("Computing the reports…");
        return;
    };
    let cal = &reports.calibration;

    egui::ScrollArea::vertical().show(ui, |ui| {
        crate::view::kit::column(ui, crate::view::kit::READING_WIDTH, |ui| {
            overview(cal, ui);
            ui.add_space(16.0);
            over_time(cal, ui);
        });
    });
}

fn overview(cal: &CalibrationReport, ui: &mut egui::Ui) {
    crate::view::kit::section(ui, "Metacognitive calibration");

    let count_text = format!(
        "Based on {} diary entries across {} reviews",
        cal.entries_used, cal.reviews_evaluated
    );
    ui.label(count_text);

    if cal.entries_used == 0 {
        ui.add_space(4.0);
        ui.weak(
            "No diary entries with self-assessment ratings yet. Record reflections after rehearsal runs to track calibration.",
        );
        return;
    }

    ui.add_space(6.0);
    ui.horizontal(|ui| {
        if let Some(s) = cal.mean_self_score {
            metric_card(ui, "Self-rated performance", format!("{:.0} %", s * 100.0));
        }
        if let Some(a) = cal.mean_accuracy {
            metric_card(ui, "Measured accuracy", format!("{:.0} %", a * 100.0));
        }
        if let Some(f) = cal.mean_familiarity {
            metric_card(ui, "Average familiarity", format!("{:.0} %", f * 100.0));
        }
        if let Some(score) = cal.score {
            metric_card(ui, "Calibration score", format!("{:.0} %", score * 100.0));
        }
    });

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.strong("Assessment:");
        ui.label(bias_label(cal.bias));
        if let Some(d) = cal.delta {
            ui.weak(format!("(gap {:+0.1} %)", d * 100.0));
        }
    });
    ui.weak(bias_explanation(cal.bias));
}

fn metric_card(ui: &mut egui::Ui, title: &str, value: String) {
    crate::view::kit::card(ui, |ui| {
        ui.vertical(|ui| {
            ui.weak(title);
            ui.label(egui::RichText::new(value).size(18.0).strong());
        });
    });
}

fn over_time(cal: &CalibrationReport, ui: &mut egui::Ui) {
    crate::view::kit::section(ui, "Performance over time");
    ui.weak("Self-rated grasp alongside measured accuracy and familiarity for each day.");

    if cal.buckets.is_empty() {
        ui.label("No timeline data recorded yet.");
        return;
    }

    ui.add_space(6.0);
    for b in cal.buckets.iter().rev() {
        bucket_row(ui, b);
        ui.separator();
    }
}

fn bucket_row(ui: &mut egui::Ui, b: &CalibrationBucket) {
    let date_str = format_date(b.date);
    ui.horizontal(|ui| {
        ui.strong(format!("{date_str}:"));
        ui.weak(format!("{} reviews, {} entries", b.reviews, b.entries));
    });

    ui.horizontal(|ui| {
        if let Some(s) = b.self_score {
            bar(ui, "Self-rating: ", s, format!("{:.0} %", s * 100.0));
        } else {
            ui.label("Self-rating: –");
        }

        if let Some(a) = b.accuracy {
            bar(ui, "Accuracy: ", a, format!("{:.0} %", a * 100.0));
        } else {
            ui.label("Accuracy: –");
        }

        if let Some(f) = b.familiarity {
            bar(ui, "Familiarity: ", f, format!("{:.0} %", f * 100.0));
        }
    });

    if let Some(g) = b.gap {
        let side = if g > 0.05 {
            "overconfident"
        } else if g < -0.05 {
            "underconfident"
        } else {
            "well calibrated"
        };
        ui.weak(format!("Gap: {:+0.1} % ({side})", g * 100.0));
    }
}
