//! True retention, young and mature retention, and the forgetting curve of each stability tier, measured
//! next to what FSRS predicts (STORY-0107). Every number is written out.

use eframe::egui;
use studyfrog_report::retention_survival::{
    RateCount, SurvivalReport, TierCurve, predicted_of, rate_of,
};

use crate::view::kit::{self, BarRow};

fn percent(x: Option<f32>) -> String {
    x.map_or("no reviews".to_owned(), |v| format!("{:.0} %", v * 100.0))
}

fn kpi_line(name: &str, c: &RateCount) -> String {
    match (rate_of(c), predicted_of(c)) {
        (Some(r), Some(p)) => format!(
            "{name}: {:.0} % of {} (FSRS expected {:.0} %)",
            r * 100.0,
            c.reviews,
            p * 100.0
        ),
        _ => format!("{name}: no reviews yet"),
    }
}

fn ratio_label(from: f32, to: f32) -> String {
    if to.is_infinite() {
        format!("t/S {from}+")
    } else {
        format!("t/S {from}-{to}")
    }
}

fn tier_label(t: &TierCurve) -> String {
    if t.to_days.is_infinite() {
        format!("Stability {}+ days", t.from_days)
    } else {
        format!("Stability {}-{} days", t.from_days, t.to_days)
    }
}

fn tier(t: &TierCurve, i: usize, ui: &mut egui::Ui) {
    let error = t.rmse.map_or(String::new(), |e| {
        format!(", off by {:.0} points (RMSE)", e * 100.0)
    });
    ui.label(format!(
        "{} ({} reviews{error})",
        tier_label(t),
        t.total.reviews
    ));
    let rows: Vec<BarRow> = t
        .bins
        .iter()
        .filter(|b| b.count.reviews > 0)
        .map(|b| BarRow {
            label: ratio_label(b.from_ratio, b.to_ratio),
            fraction: rate_of(&b.count).unwrap_or(0.0),
            text: format!(
                "{} measured, {} predicted ({})",
                percent(rate_of(&b.count)),
                percent(predicted_of(&b.count)),
                b.count.reviews
            ),
            color: None,
        })
        .collect();
    kit::bar_rows(ui, &format!("reports-tier-{i}"), &rows);
}

/// The section, under the retention by gap.
pub fn draw(s: &SurvivalReport, ui: &mut egui::Ui) {
    kit::section(ui, "True retention");
    ui.weak("Recall of mature items (stability of 21 days or more) reviewed within 20 % of their interval.");
    ui.label(kpi_line("True retention", &s.true_retention));
    ui.label(kpi_line("Young items (stability under 21 days)", &s.young));
    ui.label(kpi_line("Mature items", &s.mature));
    ui.label(kpi_line("Reviewed early", &s.early));
    ui.label(kpi_line("Reviewed on time", &s.on_time));
    ui.label(kpi_line("Overdue (resilience)", &s.overdue));
    match (s.half_life_days, s.model_half_life_days, s.memory_scale) {
        (Some(h), Some(m), Some(k)) => ui.label(format!(
            "Your memory half-life for a typical item: about {h:.0} days (FSRS predicts {m:.0}; your memory lasts {k:.2} times as long)."
        )),
        (_, Some(m), _) => ui.label(format!(
            "Memory half-life of a typical item, by FSRS: about {m:.0} days (too few reviews to measure your own)."
        )),
        _ => ui.weak("No half-life yet."),
    };
    kit::section(ui, "Forgetting curves by stability");
    ui.weak("Recall by the time since the previous review relative to the stability (t/S), measured and predicted.");
    let mut any = false;
    for (i, t) in s
        .tiers
        .iter()
        .enumerate()
        .filter(|(_, t)| t.total.reviews > 0)
    {
        tier(t, i, ui);
        any = true;
    }
    if !any {
        ui.label("No spaced reviews yet.");
    }
}
