//! The Habits tab of the reports: recall and answer time by local hour and weekday with the peak and trough
//! (STORY-0102), and how recall and pace change within a run (STORY-0103). Every bar carries its numbers as text.

use eframe::egui;
use studyfrog_report::chronotype::{
    ChronotypeReport, ClockBucket, ClockWindow, MIN_BUCKET_REVIEWS, with_windows,
};
use studyfrog_report::fatigue::{FatigueReport, PaceBucket, pace_rate};

use crate::explorer_ui::ExplorerMsg;
use crate::msg::Msg;
use crate::state::State;
use crate::view::kit::{self, READING_WIDTH};
use crate::view::report_rows::{DetailRow, detail_rows};

const WEEKDAYS: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];
/// The thresholds offered for the peak and trough.
pub const THRESHOLDS: [u32; 4] = [5, 10, 20, 50];

fn seconds(ms: Option<u32>) -> String {
    ms.map_or("no answer times".to_owned(), |m| {
        format!("median {:.1} s", f64::from(m) / 1000.0)
    })
}

fn rate(reviews: u32, recalled: u32) -> f32 {
    recalled as f32 / reviews.max(1) as f32
}

fn percent(x: f32) -> String {
    format!("{:.0} %", x * 100.0)
}

fn clock_detail(b: &ClockBucket) -> String {
    let p90 = b.p90_ms.map_or(String::new(), |m| {
        format!(", 90 % under {:.1} s", f64::from(m) / 1000.0)
    });
    format!("of {}, {}{p90}", b.reviews, seconds(b.median_ms))
}

fn clock_row(label: String, b: &ClockBucket) -> DetailRow {
    let r = rate(b.reviews, b.recalled);
    DetailRow {
        label,
        fraction: r,
        value: percent(r),
        detail: clock_detail(b),
    }
}

fn window_text(w: &ClockWindow) -> String {
    let end = (u32::from(w.start_hour) + u32::from(w.hours)) % 24;
    format!(
        "between {:02}:00 and {end:02}:00 ({:.0} % of {}, {})",
        w.start_hour,
        w.retention * 100.0,
        w.reviews,
        seconds(w.median_ms)
    )
}

/// The tab.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(r) = &state.reports else {
        ui.label("Computing the reports…");
        return;
    };
    let a = &r.analytics;
    let min = match state.explorer.min_bucket {
        0 => MIN_BUCKET_REVIEWS,
        n => n,
    };
    let chrono = with_windows(a.chronotype.clone(), min);
    egui::ScrollArea::vertical().show(ui, |ui| {
        kit::column(ui, READING_WIDTH, |ui| {
            clock(&chrono, min, ui, out);
            ui.add_space(16.0);
            fatigue(&a.fatigue, ui);
        });
    });
}

fn threshold(min: u32, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal_wrapped(|ui| {
        ui.label("Reviews an hour needs before it is recommended:");
        for n in THRESHOLDS {
            if ui.selectable_label(min == n, n.to_string()).clicked() {
                out.push(Msg::Explorer(ExplorerMsg::MinBucket(n)));
            }
        }
    });
}

fn clock(c: &ChronotypeReport, min: u32, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    kit::section(ui, "When you remember best");
    ui.weak("Recall and time to answer by the local hour of the review.");
    threshold(min, ui, out);
    match (&c.peak, &c.trough) {
        (Some(p), trough) => {
            ui.label(format!("Your recall is highest {}.", window_text(p)));
            match trough {
                Some(t) => ui.label(format!("It is lowest {}.", window_text(t))),
                None => {
                    ui.label("No other two hours in a row have enough reviews to name a low point.")
                }
            };
        }
        (None, _) => {
            ui.label(format!(
                "Not enough reviews yet: two hours in a row need {min} reviews each before one is recommended."
            ));
        }
    };
    let rows: Vec<DetailRow> = c
        .hours
        .iter()
        .enumerate()
        .filter(|(_, b)| b.reviews > 0)
        .map(|(h, b)| clock_row(format!("{h:02}:00"), b))
        .collect();
    detail_rows(ui, "reports-hours", &rows);
    weekdays(c, ui);
}

fn weekdays(c: &ChronotypeReport, ui: &mut egui::Ui) {
    kit::section(ui, "By weekday");
    if let (Some(b), Some(w)) = (c.best_weekday, c.worst_weekday) {
        ui.label(format!(
            "Best on {}, weakest on {}.",
            WEEKDAYS[b], WEEKDAYS[w]
        ));
    }
    let rows: Vec<DetailRow> = c
        .weekdays
        .iter()
        .enumerate()
        .filter(|(_, b)| b.reviews > 0)
        .map(|(d, b)| clock_row(WEEKDAYS[d].to_owned(), b))
        .collect();
    if rows.is_empty() {
        ui.label("No reviews yet.");
    }
    detail_rows(ui, "reports-weekdays", &rows);
}

fn pace_text(b: &PaceBucket) -> String {
    format!(
        "{:.0} % of {}, {}",
        pace_rate(b).unwrap_or(0.0) * 100.0,
        b.reviews,
        seconds(b.median_ms)
    )
}

fn range(b: &PaceBucket, unit: &str) -> String {
    match b.to {
        u32::MAX => format!("{}+ {unit}", b.from),
        to if unit == "min" => format!("{}-{to} {unit}", b.from),
        to => format!("{unit} {}-{}", b.from, to - 1),
    }
}

fn pace_rows(buckets: &[PaceBucket], unit: &str) -> Vec<DetailRow> {
    buckets
        .iter()
        .filter(|b| b.reviews > 0)
        .map(|b| {
            let r = pace_rate(b).unwrap_or(0.0);
            DetailRow {
                label: range(b, unit),
                fraction: r,
                value: percent(r),
                detail: format!("of {}, {}", b.reviews, seconds(b.median_ms)),
            }
        })
        .collect()
}

fn fatigue(f: &FatigueReport, ui: &mut egui::Ui) {
    kit::section(ui, "Within a run");
    ui.weak("Recall and time to reveal by how long the run had been going (pauses left out) and by the item's place in it.");
    match &f.inflection {
        Some(i) => {
            let items = i
                .items
                .map_or(String::new(), |n| format!(" (about {n} items)"));
            ui.label(format!(
                "Your accuracy holds for about {} minutes{items}; after that the median time to answer rises {:.0} % and recall drops {:.0} points. Runs of about {} minutes suit you.",
                i.minutes,
                i.latency_rise.max(0.0) * 100.0,
                i.recall_drop_points.max(0.0),
                i.minutes
            ));
        }
        None if f.sessions > 0 => {
            ui.label(format!(
                "No point of fatigue found in {} run(s): recall and pace hold steady.",
                f.sessions
            ));
        }
        None => {
            ui.label("No runs recorded yet.");
        }
    }
    ui.label(format!("First ten minutes: {}", pace_text(&f.baseline)));
    detail_rows(ui, "reports-fatigue-time", &pace_rows(&f.by_time, "min"));
    ui.add_space(8.0);
    detail_rows(
        ui,
        "reports-fatigue-position",
        &pace_rows(&f.by_position, "items"),
    );
}
