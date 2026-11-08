//! The Workload tab of the reports: the simulated reviews or study minutes ahead by stage with their band, for
//! 30, 90 or 365 days, and the exam pacer of every stack with an exam ahead (STORY-0105). Every number is text.

use eframe::egui;
use studyfrog_report::forecast_sim::{DayLoad, load_total};
use studyfrog_service::analytics_plan::Plan;

use crate::explorer_ui::ExplorerMsg;
use crate::msg::Msg;
use crate::state::State;
use crate::view::kit::{self, READING_WIDTH};
use crate::view::report_rows::{DetailRow, detail_rows};

/// The horizons offered.
pub const HORIZONS: [u16; 3] = [30, 90, 365];

/// Days shown for a stored choice.
pub fn horizon_of(stored: u16) -> usize {
    usize::from(if stored == 0 { 30 } else { stored })
}

/// Days grouped into one row: one for the first week of 30 days, then weeks, then four weeks for a year.
fn rows_of(days: usize) -> Vec<(usize, usize)> {
    let step = if days > 90 { 28 } else { 7 };
    let mut out: Vec<(usize, usize)> = Vec::new();
    if days <= 30 {
        out.extend((0..7.min(days)).map(|d| (d, d + 1)));
    }
    let mut from = if days <= 30 { 7 } else { 0 };
    while from < days {
        out.push((from, (from + step).min(days)));
        from += step;
    }
    out
}

fn label(from: usize, to: usize) -> String {
    match (from, to - from) {
        (0, 1) => "Today".to_owned(),
        (1, 1) => "Tomorrow".to_owned(),
        (d, 1) => format!("In {d} days"),
        (f, _) => format!("Days {f}-{}", to - 1),
    }
}

fn sum(days: &[DayLoad], f: fn(&DayLoad) -> f32) -> f32 {
    days.iter().map(f).sum()
}

fn row(days: &[DayLoad], from: usize, to: usize, minutes: bool, most: f32) -> DetailRow {
    let d = &days[from..to];
    let (value, typical, low, high) = if minutes {
        (
            sum(d, |x| x.minutes),
            sum(d, |x| x.minutes),
            sum(d, |x| x.minutes_p10),
            sum(d, |x| x.minutes_p90),
        )
    } else {
        let p50 = sum(d, |x| x.p50);
        (
            sum(d, load_total),
            p50,
            sum(d, |x| x.p10),
            sum(d, |x| x.p90),
        )
    };
    let unit = if minutes { "min" } else { "reviews" };
    DetailRow {
        label: label(from, to),
        fraction: if most > 0.0 { value / most } else { 0.0 },
        value: format!("{value:.0} {unit}"),
        detail: format!(
            "new {:.0}, learning {:.0}, young {:.0}, mature {:.0}; typical {typical:.0}, {low:.0} to {high:.0} {unit} in a low or busy future",
            sum(d, |x| x.new),
            sum(d, |x| x.learning),
            sum(d, |x| x.young),
            sum(d, |x| x.mature)
        ),
    }
}

fn choices(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let days = horizon_of(state.explorer.horizon);
    ui.horizontal_wrapped(|ui| {
        ui.label("Show:");
        for h in HORIZONS {
            if ui
                .selectable_label(days == usize::from(h), format!("{h} days"))
                .clicked()
            {
                out.push(Msg::Explorer(ExplorerMsg::Horizon(h)));
            }
        }
        ui.separator();
        for (on, name) in [(false, "Reviews"), (true, "Study minutes")] {
            if ui
                .selectable_label(state.explorer.minutes == on, name)
                .clicked()
            {
                out.push(Msg::Explorer(ExplorerMsg::Minutes(on)));
            }
        }
    });
}

fn forecast(plan: &Plan, state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    kit::section(ui, "Workload ahead");
    ui.weak(format!(
        "Each card's FSRS or SM-2 state carried forward in 16 simulated futures, with your ratings, {:.0} % of days missed and {:.1} new card(s) a day.",
        plan.miss_probability * 100.0,
        plan.new_per_day
    ));
    choices(state, ui, out);
    let days = horizon_of(state.explorer.horizon).min(plan.forecast.len());
    let f = &plan.forecast[..days];
    ui.label(format!(
        "About {:.0} reviews and {:.0} minutes of study in the next {days} days.",
        sum(f, load_total),
        sum(f, |x| x.minutes)
    ));
    let minutes = state.explorer.minutes;
    let spans = rows_of(days);
    let value = |a: usize, b: usize| {
        if minutes {
            sum(&f[a..b], |x| x.minutes)
        } else {
            sum(&f[a..b], load_total)
        }
    };
    let most = spans
        .iter()
        .map(|(a, b)| value(*a, *b))
        .fold(0.0_f32, f32::max);
    let rows: Vec<DetailRow> = spans
        .iter()
        .map(|(a, b)| row(f, *a, *b, minutes, most))
        .collect();
    detail_rows(ui, "reports-workload", &rows);
}

/// The tab.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(r) = &state.reports else {
        ui.label("Computing the reports…");
        return;
    };
    let plan = &r.analytics.plan;
    egui::ScrollArea::vertical().show(ui, |ui| {
        kit::column(ui, READING_WIDTH, |ui| {
            forecast(plan, state, ui, out);
            kit::section(ui, "Exam pacer");
            if plan.pacers.is_empty() {
                ui.label("No stack has an exam or due date ahead. Set one on the stack page or in the calendar.");
            }
            for p in &plan.pacers {
                crate::view::report_pacer::draw(p, ui);
            }
        });
    });
}
