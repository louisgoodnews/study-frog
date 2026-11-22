//! The memory game's start from the calendar's rows: a session's stack, or a planned review's stack.

use eframe::egui;
use studyfrog_core::uid::Uid;

use crate::msg::Msg;
use crate::planner::PlanMsg;

/// A "Memory game for <title>" button for `stack`. Nothing is drawn for a row without a stack.
pub fn offer(ui: &mut egui::Ui, stack: Option<Uid>, title: &str, out: &mut Vec<Msg>) {
    let Some(stack) = stack else {
        return;
    };
    if ui
        .button(format!("Memory game for {title}"))
        .on_hover_text(
            "Plays a memory game with the cards of this stack, with the timing chosen in Rehearse.",
        )
        .clicked()
    {
        out.push(Msg::Plan(PlanMsg::Memory(stack)));
    }
}
