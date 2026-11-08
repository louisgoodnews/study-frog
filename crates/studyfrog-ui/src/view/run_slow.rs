//! A quiet line during a run when the answer just revealed took far longer than the run's usual, and the setting
//! that turns it off (STORY-0103). It never blocks or interrupts.

use eframe::egui;
use studyfrog_core::event::LatencyKind;
use studyfrog_rehearsal::session::{ItemPhase, current_item};
use studyfrog_report::fatigue::latency_spike;

use crate::app_settings_ui::SettingsTabMsg;
use crate::msg::Msg;
use crate::run_state::RunView;
use crate::state::State;

/// How many times slower than the run's usual the answer on screen was, when it is a spike.
pub fn slow_factor(run: &RunView) -> Option<f32> {
    if run.session.phase != ItemPhase::Revealed {
        return None;
    }
    let item = current_item(&run.session)?;
    let reveals: Vec<_> = run
        .latencies
        .iter()
        .filter(|l| matches!(l.kind, LatencyKind::Flip | LatencyKind::Answer))
        .collect();
    let (last, earlier) = reveals.split_last()?;
    if last.item != item {
        return None;
    }
    let earlier: Vec<u32> = earlier.iter().map(|l| l.ms).collect();
    latency_spike(&earlier, last.ms)
}

/// The line under the card, when it applies and is not switched off.
pub fn notice(state: &State, run: &RunView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if state.app.slow_answer_notice_off {
        return;
    }
    let Some(factor) = slow_factor(run) else {
        return;
    };
    ui.horizontal_wrapped(|ui| {
        ui.weak(format!(
            "That one took {factor:.1} times as long as your usual answer in this run. A short break can help."
        ));
        if ui.small_button("Don't show this").clicked() {
            out.push(Msg::SettingsTab(SettingsTabMsg::SlowAnswerNotice(false)));
        }
    });
}

/// The setting on the Rehearsal tab of the settings.
pub fn setting(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    crate::view::kit::section(ui, "Slow answers");
    let mut on = !state.app.slow_answer_notice_off;
    if ui
        .checkbox(
            &mut on,
            "Say when an answer takes far longer than usual during a run",
        )
        .changed()
    {
        out.push(Msg::SettingsTab(SettingsTabMsg::SlowAnswerNotice(on)));
    }
    ui.weak("At least twice the median of the run's earlier answers and five seconds more; a quiet line under the card.");
}
