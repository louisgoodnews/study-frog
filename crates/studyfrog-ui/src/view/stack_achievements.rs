//! A stack's achievement summary on its page: how many are unlocked, linking to them.

use eframe::egui;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;

use crate::achievements::{AchievementsMsg, Filter, stack_counts};
use crate::msg::Msg;
use crate::state::{Screen, State};

/// The summary of the stack with this uid. It asks for the report once when there is none yet, and
/// shows nothing for a stack without achievements.
pub fn badge(state: &State, stack: Uid, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(report) = &state.achievements else {
        if !state.achievements_wanted {
            out.push(Msg::Achievements(AchievementsMsg::Wanted));
        }
        return;
    };
    let key = format_ulid(stack);
    let (done, total) = stack_counts(&report.entries, &key);
    if total == 0 {
        return;
    }
    let text = format!("{done} / {total} achievements unlocked");
    if ui
        .link(text)
        .on_hover_text("Show this stack's achievements")
        .clicked()
    {
        out.push(Msg::Achievements(AchievementsMsg::Filter(Filter::Stack(
            key,
        ))));
        out.push(Msg::Navigate(Screen::Achievements));
    }
}
