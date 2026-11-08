//! The workload-versus-retention curve with its recommendation and the button that applies it, for the vault
//! (Reports > Workload) and for a stack (its Reports tab) (STORY-0106). Every number is text.

use eframe::egui;
use studyfrog_report::cmrr::{CmrrPoint, CmrrReport};

use crate::explorer_ui::ExplorerMsg;
use crate::msg::Msg;
use crate::view::report_rows::{DetailRow, detail_rows};

fn marks(x: &CmrrPoint, r: &CmrrReport, current: f32) -> String {
    let near = |a: f32, b: f32| (a - b).abs() < 0.005;
    let mut m: Vec<&str> = Vec::new();
    if r.recommended.is_some_and(|v| near(v, x.retention)) {
        m.push("recommended");
    }
    if r.min_workload_retention
        .is_some_and(|v| near(v, x.retention))
    {
        m.push("least work");
    }
    if near(current, x.retention) {
        m.push("now");
    }
    if m.is_empty() {
        String::new()
    } else {
        format!(" ({})", m.join(", "))
    }
}

fn rows(r: &CmrrReport, current: f32) -> Vec<DetailRow> {
    let most = r
        .points
        .iter()
        .map(|x| x.daily_minutes)
        .fold(0.0_f32, f32::max);
    r.points
        .iter()
        .map(|x| DetailRow {
            label: format!("{:.2}{}", x.retention, marks(x, r, current)),
            fraction: if most > 0.0 {
                x.daily_minutes / most
            } else {
                0.0
            },
            value: format!("{:.1} min/day", x.daily_minutes),
            detail: format!(
                "{:.0} reviews a day, {:.0} cards remembered ({:.0} %), {:.1} per minute",
                x.daily_reviews,
                x.memorized,
                x.mean_retention * 100.0,
                x.efficiency
            ),
        })
        .collect()
}

/// The curve of `report` for the vault (`folder` `None`) or a stack, named `name`, whose retention is `current`.
pub fn draw(
    report: &CmrrReport,
    current: f32,
    folder: Option<&str>,
    name: &str,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    ui.weak(format!(
        "Each desired retention simulated over {} days with your cards, ratings and pace: the time a day it costs and how much you would remember.",
        report.days
    ));
    let Some(best) = report.recommended else {
        ui.label("Nothing to simulate yet: there are no cards with a history.");
        return;
    };
    let least = report.min_workload_retention.unwrap_or(best);
    let lowest = studyfrog_report::cmrr::RETENTION_GRID[0];
    let floor = if least > lowest + 0.005 {
        format!("Below {least:.2} forgetting adds more relearning than it saves.")
    } else {
        format!(
            "Down to {lowest:.2} a lower target still saves time here, at the cost of remembering less."
        )
    };
    ui.label(format!(
        "Recommended for {name}: {best:.2}, the most cards remembered per minute of study (now {current:.2}). {floor}"
    ));
    detail_rows(
        ui,
        &format!("cmrr-{}", folder.unwrap_or("vault")),
        &rows(report, current),
    );
    let label = format!("Apply recommended retention {best:.2} to {name}");
    if ui
        .add_enabled((best - current).abs() >= 0.005, egui::Button::new(label))
        .clicked()
    {
        let permille = (best * 1000.0).round() as u16;
        out.push(Msg::Explorer(ExplorerMsg::ApplyRetention(
            folder.map(str::to_owned),
            permille,
        )));
    }
}
