//! The Roadmap tab: rows with status and bars on a time axis with a line for today, scrolling both ways. Only the rows in
//! view are built; each bar is a button that opens its stack or item and says its dates in text.

use eframe::egui::{self, Align2, FontId, Rect, Role, Sense, Stroke, WidgetInfo, pos2, vec2};
use studyfrog_core::civil::{date_from_days, format_date};
use studyfrog_core::status::status_label;

use crate::library_ui::LibraryMsg;
use crate::msg::Msg;
use crate::stack_roadmap::{RoadRow, ZOOMS, axis, bar, px_per_day, rows, tick_days, visible};
use crate::stack_view_ui::StackViewMsg;
use crate::state::{Selection, State};
use crate::theme::{palette_of, tint};

const ROW_H: f32 = 38.0;
const HEAD_H: f32 = 28.0;
const BAR_H: f32 = 18.0;

fn describe(r: &RoadRow) -> String {
    let kind = if r.folder.is_some() { "Stack" } else { "Item" };
    let (a, b) = (
        format_date(date_from_days(r.start)),
        format_date(date_from_days(r.end)),
    );
    format!("{kind} {}, {}, {a} to {b}", r.title, status_label(r.status))
}

fn zoom_bar(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal(|ui| {
        ui.label("Zoom");
        for (z, label) in ZOOMS {
            if ui
                .selectable_label(state.library.stack_view.zoom == z, label)
                .clicked()
            {
                out.push(Msg::Library(LibraryMsg::Stack(StackViewMsg::Zoom(z))));
            }
        }
    });
}

fn axis_ticks(
    ui: &egui::Ui,
    canvas: Rect,
    (from, to): (i64, i64),
    (ppd, step): (f32, i64),
    view_x: (f32, f32),
) {
    let pal = palette_of(ui);
    let first = from - from.rem_euclid(step) + step;
    let mut day = first;
    while day <= to {
        let x = canvas.left() + (day - from) as f32 * ppd;
        if x >= view_x.0 - 80.0 && x <= view_x.1 {
            let text = format_date(date_from_days(day));
            ui.painter().text(
                pos2(x + 2.0, canvas.top() + 2.0),
                Align2::LEFT_TOP,
                text,
                FontId::proportional(12.0),
                pal.text_weak,
            );
            ui.painter().line_segment(
                [pos2(x, canvas.top()), pos2(x, canvas.bottom())],
                Stroke::new(0.5, tint(pal.border, 120)),
            );
        }
        day += step;
    }
}

/// The tab.
pub fn draw(state: &State, folder: &str, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(vault) = &state.vault else { return };
    let today = state.wall_ms.div_euclid(86_400_000);
    let list = rows(&state.library, &vault.overview.stacks, folder, today);
    zoom_bar(state, ui, out);
    if list.is_empty() {
        ui.weak("Nothing to show yet: the roadmap lists the stacks inside this one and its items.");
        return;
    }
    let z = state.library.stack_view.zoom;
    let (from, to) = axis(&list, today);
    // The chart takes all the room there is: a short span is stretched to the width, a short list to the height.
    let span = (to - from).max(1) as f32;
    let ppd = px_per_day(z).max((ui.available_width() - 24.0) / span);
    let height = ui.available_height().max(240.0);
    let size = vec2(
        span * ppd,
        (HEAD_H + list.len() as f32 * ROW_H).max(height - 12.0),
    );
    let pal = palette_of(ui);
    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .max_height(height)
        .show_viewport(ui, |ui, view| {
            let (canvas, _) = ui.allocate_exact_size(size, Sense::hover());
            axis_ticks(
                ui,
                canvas,
                (from, to),
                (ppd, tick_days(z)),
                (canvas.left() + view.left(), canvas.left() + view.right()),
            );
            let rows_in_view = visible(
                list.len(),
                ROW_H,
                (view.top() - HEAD_H).max(0.0),
                view.height(),
            );
            for i in rows_in_view {
                let r = &list[i];
                let (x, w) = bar(r, from, ppd);
                let top = canvas.top() + HEAD_H + i as f32 * ROW_H;
                let rect = Rect::from_min_size(pos2(canvas.left() + x, top + 17.0), vec2(w, BAR_H));
                let hit = ui.interact(rect, ui.scope_id().with(("road", i)), Sense::click());
                let color = if r.folder.is_some() {
                    pal.accent
                } else {
                    pal.success
                };
                ui.painter().rect_filled(
                    rect,
                    3.0,
                    tint(color, if hit.hovered() { 200 } else { 140 }),
                );
                let label_x = (canvas.left() + x).max(canvas.left() + view.left() + 4.0);
                let text = format!("{} - {}", r.title, status_label(r.status));
                ui.painter().text(
                    pos2(label_x, top + 1.0),
                    Align2::LEFT_TOP,
                    text,
                    FontId::proportional(13.0),
                    pal.text,
                );
                let name = describe(r);
                hit.widget_info(|| WidgetInfo::labeled(Role::Button, true, name.clone()));
                if hit.clicked() {
                    match (&r.folder, r.uid) {
                        (Some(f), _) => out.push(Msg::Select(Selection::Stack(f.clone()))),
                        (None, Some(u)) => out.push(Msg::OpenItem(u)),
                        _ => {}
                    }
                }
            }
            let x = canvas.left() + (today - from) as f32 * ppd;
            ui.painter().line_segment(
                [pos2(x, canvas.top()), pos2(x, canvas.bottom())],
                Stroke::new(1.5, pal.danger),
            );
            ui.painter().text(
                pos2(x + 3.0, canvas.top() + 12.0),
                Align2::LEFT_TOP,
                "Today",
                FontId::proportional(12.0),
                pal.danger,
            );
        });
}
