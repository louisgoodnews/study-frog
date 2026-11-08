//! The reviews the schedule planned, on the calendar: chips in the day cells that can be dragged to another day,
//! and the list under the chosen day with a date field, for those who do not drag.

use eframe::egui::{self, Align2, Pos2, Rect, Role, Sense, Stroke, WidgetInfo, vec2};
use studyfrog_calendar::review_plan::PlanEntry;
use studyfrog_core::civil::{Date, date_to_days};
use studyfrog_core::uid::Uid;

use crate::msg::Msg;
use crate::planner::PlanMsg;
use crate::planner_reviews::ReviewMsg;
use crate::state::State;
use crate::theme::{palette_of, tint};
use crate::view::{date_picker, kit};

/// A chip let go of: which entry, and where.
pub struct Dropped {
    stack: Option<Uid>,
    from: i64,
    at: Pos2,
}

fn send(out: &mut Vec<Msg>, m: ReviewMsg) {
    out.push(Msg::Plan(PlanMsg::Reviews(m)));
}

/// The stack's title, or "Inbox".
pub fn title_of(state: &State, stack: Option<Uid>) -> String {
    let found = stack.and_then(|u| {
        let stacks = &state.vault.as_ref()?.overview.stacks;
        stacks.iter().find(|s| s.uid == u).map(|s| s.title.clone())
    });
    found.unwrap_or_else(|| "Inbox".to_owned())
}

fn on_day(state: &State, day: Date, scope: Option<Uid>) -> Vec<&PlanEntry> {
    let n = date_to_days(day);
    state
        .planner
        .reviews
        .iter()
        .filter(|e| e.day == n && scope.is_none_or(|u| e.stack == Some(u)))
        .collect()
}

fn line(state: &State, e: &PlanEntry) -> String {
    format!(
        "{}: {} of {} due",
        title_of(state, e.stack),
        e.items.len(),
        e.stack_size
    )
}

fn chip(state: &State, e: &PlanEntry, rect: Rect, ui: &mut egui::Ui) -> egui::Response {
    let pal = palette_of(ui);
    let id = ui.scope_id().with(("review-chip", e.stack, e.day));
    let r = ui.interact(rect, id, Sense::click_and_drag());
    let fill = if r.hovered() {
        tint(pal.accent, 50)
    } else {
        tint(pal.accent, 20)
    };
    ui.painter().rect(
        rect,
        3.0,
        fill,
        Stroke::new(1.0, pal.accent),
        egui::StrokeKind::Inside,
    );
    ui.painter().with_clip_rect(rect).text(
        rect.left_center() + vec2(4.0, 0.0),
        Align2::LEFT_CENTER,
        line(state, e),
        crate::typo::font(ui, crate::typo::Tier::Caption),
        pal.text,
    );
    let name = format!("Planned reviews, {}", line(state, e));
    r.widget_info(|| WidgetInfo::labeled(Role::Button, true, name.clone()));
    r
}

/// The chips of the planned reviews of `day`, from line `first` down. Returns the chips let go of.
pub fn day_chips(
    state: &State,
    day: Date,
    (cell, first): (Rect, usize),
    scope: Option<Uid>,
    (ui, out): (&mut egui::Ui, &mut Vec<Msg>),
) -> Vec<Dropped> {
    let mut dropped = Vec::new();
    for (i, e) in on_day(state, day, scope).into_iter().enumerate() {
        let top = cell.top() + 24.0 + (first + i) as f32 * 17.0;
        let rect = Rect::from_min_size(
            egui::pos2(cell.left() + 4.0, top),
            vec2(cell.width() - 8.0, 15.0),
        );
        if rect.bottom() > cell.bottom() {
            break;
        }
        let r = chip(state, e, rect, ui);
        if r.clicked() {
            send(out, ReviewMsg::OpenStack(e.stack));
        }
        if r.drag_stopped()
            && let Some(at) = ui.input(|i| i.pointer.interact_pos())
        {
            dropped.push(Dropped {
                stack: e.stack,
                from: e.day,
                at,
            });
        }
        if r.dragged() {
            r.clone()
                .on_hover_text(format!("Drop on a day to move {}", line(state, e)));
        }
        r.context_menu(|ui| {
            if ui.button("Move to another day…").clicked() {
                out.push(Msg::Plan(PlanMsg::Select(day)));
                send(out, ReviewMsg::OpenMove(e.stack, e.day));
                ui.close();
            }
        });
    }
    dropped
}

/// Turn the chips let go of into moves, by the day cell under the pointer.
pub fn resolve(dropped: &[Dropped], cells: &[(Date, Rect)], out: &mut Vec<Msg>) {
    for d in dropped {
        if let Some((day, _)) = cells.iter().find(|(_, rect)| rect.contains(d.at)) {
            let to = date_to_days(*day);
            send(
                out,
                ReviewMsg::Drop {
                    stack: d.stack,
                    from: d.from,
                    to,
                },
            );
        }
    }
}

fn entry_card(state: &State, e: &PlanEntry, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let title = title_of(state, e.stack);
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(line(state, e));
        ui.horizontal_wrapped(|ui| {
            if ui.button(format!("Open {title}")).clicked() {
                send(out, ReviewMsg::OpenStack(e.stack));
            }
            if ui.button(format!("Move {title} reviews…")).clicked() {
                send(out, ReviewMsg::OpenMove(e.stack, e.day));
            }
        });
        let Some(form) = state
            .planner
            .moving
            .as_ref()
            .filter(|f| f.stack == e.stack && f.day == e.day)
        else {
            return;
        };
        let today = date_picker::today_of(state);
        if let Some(text) = date_picker::date_field(ui, "Move reviews to", &form.text, today) {
            send(out, ReviewMsg::MoveText(text));
        }
        ui.horizontal(|ui| {
            if ui.button("Move").clicked() {
                send(out, ReviewMsg::Confirm);
            }
            if ui.button("Cancel move").clicked() {
                send(out, ReviewMsg::Cancel);
            }
        });
    });
}

/// The section under the chosen day: the planned reviews of that day, the setting and the undo.
pub fn side_section(state: &State, scope: Option<Uid>, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let mut on = crate::planner_reviews::shown(state);
    if ui
        .checkbox(&mut on, "Show the reviews the schedule plans")
        .changed()
    {
        send(out, ReviewMsg::Show(on));
    }
    if state.planner.undo.is_some() && ui.button("Undo last move").clicked() {
        send(out, ReviewMsg::Undo);
    }
    if !on {
        return;
    }
    let entries = on_day(state, state.planner.selected, scope);
    if entries.is_empty() {
        ui.weak("No reviews planned for this day.");
    }
    for e in entries {
        entry_card(state, e, ui, out);
    }
}
