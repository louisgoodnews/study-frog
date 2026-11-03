//! The tiles, rating bars and activity chart of the performance explorer.

use eframe::egui::{self, Rect, RichText, Role, Sense, WidgetInfo, vec2};
use studyfrog_report::explore::{DayPoint, Perf};

use crate::theme::{palette_of, tint};
use crate::view::explorer::{day_of, minutes, percent, seconds};
use crate::view::kit;

pub(super) fn tiles(p: &Perf, ui: &mut egui::Ui) {
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            kit::stat(ui, &p.reviews.to_string(), "Reviews");
            ui.add_space(18.0);
            kit::stat(ui, &p.items.to_string(), "Items reviewed");
            ui.add_space(18.0);
            kit::stat(ui, &percent(p), "Good or easy");
            ui.add_space(18.0);
            kit::stat(
                ui,
                &p.median_ms.map_or("-".to_owned(), seconds),
                "Median time to answer",
            );
            ui.add_space(18.0);
            kit::stat(ui, &minutes(p.total_ms), "Time answering");
            ui.add_space(18.0);
            kit::stat(ui, &day_of(p.last_ms), "Last review");
        });
    });
}

pub(super) fn ratings(p: &Perf, ui: &mut egui::Ui) {
    let pal = palette_of(ui);
    let colors = [pal.danger, pal.warning, pal.success, pal.accent];
    kit::section(ui, "How the reviews were rated");
    let rows: Vec<kit::BarRow> = ["Again", "Hard", "Good", "Easy"]
        .into_iter()
        .enumerate()
        .map(|(i, name)| kit::BarRow {
            label: format!("{name}: {}", p.ratings[i]),
            fraction: if p.reviews == 0 {
                0.0
            } else {
                p.ratings[i] as f32 / p.reviews as f32
            },
            text: String::new(),
            color: Some(colors[i]),
        })
        .collect();
    kit::bar_rows(ui, "explorer-ratings", &rows);
}

/// Bars for the last days: the height is the number of reviews, the lower part the good ones.
pub(super) fn chart(days: &[DayPoint], ui: &mut egui::Ui) {
    kit::section(ui, "Last 30 days");
    let total: u32 = days.iter().map(|d| d.reviews).sum();
    let active = days.iter().filter(|d| d.reviews > 0).count();
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 90.0), Sense::hover());
    response.widget_info(|| {
        WidgetInfo::labeled(
            Role::Image,
            true,
            format!(
                "{total} reviews on {active} of the last {} days",
                days.len()
            ),
        )
    });
    let pal = palette_of(ui);
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 6.0, pal.surface);
    let most = days.iter().map(|d| d.reviews).max().unwrap_or(0).max(1) as f32;
    let width = rect.width() / days.len().max(1) as f32;
    for (i, d) in days.iter().enumerate() {
        if d.reviews == 0 {
            continue;
        }
        let height = (rect.height() - 8.0) * d.reviews as f32 / most;
        let left = rect.left() + i as f32 * width + 1.0;
        let bar = Rect::from_min_max(
            egui::pos2(left, rect.bottom() - 4.0 - height),
            egui::pos2(left + width - 2.0, rect.bottom() - 4.0),
        );
        painter.rect_filled(bar, 2.0, tint(pal.warning, 160));
        let good = bar.height() * d.good as f32 / d.reviews as f32;
        painter.rect_filled(
            Rect::from_min_max(
                egui::pos2(bar.left(), bar.bottom() - good),
                bar.right_bottom(),
            ),
            2.0,
            pal.success,
        );
    }
    ui.label(
        RichText::new(format!(
            "{total} reviews on {active} days; green: good or easy"
        ))
        .small()
        .color(pal.text_weak),
    );
}
