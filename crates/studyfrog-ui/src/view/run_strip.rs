//! The rehearsal screen's top strip: position, mode, timer, the progress bar and the buttons.

use eframe::egui::{self, RichText};
use studyfrog_core::run::{RunMode, RunState};
use studyfrog_rehearsal::input::Action;
use studyfrog_rehearsal::session::ItemPhase;

use crate::msg::Msg;
use crate::run_state::{RunView, engine_ms};
use crate::state::State;
use crate::theme::palette_of;
use crate::view::kit;
use crate::view::run::act;

/// Position, mode, timer and the buttons, with the progress bar under them. The items are one wrapped row, so at
/// a narrow window the buttons move to a second line instead of drawing over the position and the timer.
pub(super) fn top_strip(state: &State, run: &RunView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let s = &run.session;
    let weak = palette_of(ui).text_weak;
    let position = (s.cursor + 1).min(s.queue.len());
    ui.horizontal_wrapped(|ui| {
        let total = s.queue.len();
        let count = format!("{position} / {total}");
        let widest = crate::typo::figure_template(&format!("{total} / {total}"));
        let font = crate::typo::font(ui, crate::typo::Tier::Heading);
        let min = crate::typo::text_width(ui.ctx(), &widest, &font);
        ui.add_sized(
            [min, ui.spacing().interact_size.y],
            egui::Label::new(RichText::new(count).font(font).strong()),
        );
        let mode = match s.run.mode {
            RunMode::Answer => "Type answers",
            RunMode::Exam => "Exam",
            RunMode::Flip => "Flip cards",
        };
        kit::pill(ui, mode, weak);
        if s.run.state == RunState::Running && s.phase != ItemPhase::NoItem {
            let now = engine_ms(state.now, run.started_time, s.last_t_ms);
            let on_item = now
                .saturating_sub(s.shown_at_ms)
                .saturating_sub(s.paused_in_item_ms);
            let secs = format!("{:.1} s", f64::from(on_item) / 1000.0);
            kit::figure_pill(ui, &secs, weak, "000.0 s");
        }
        if let Some(clock) = crate::run_clock::clock_of(state, run) {
            crate::view::run_timer::draw(&clock, ui);
        }
        strip_buttons(run, ui, out);
    });
    let p = crate::run_progress::progress(s);
    kit::progress_with_text(
        ui,
        crate::run_progress::fraction(p),
        &crate::run_progress::text(p),
        "Run progress",
    );
}

/// Comments, Edit, Pause and Stop, in that order from left to right.
fn strip_buttons(run: &RunView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let s = &run.session;
    let comment_count = studyfrog_rehearsal::session::current_item(s)
        .and_then(|uid| run.content.get(&uid))
        .map(|c| c.comments.len())
        .unwrap_or(0);
    let comments_label = if comment_count > 0 {
        format!("Comments ({comment_count})")
    } else {
        "Comments".to_string()
    };
    if ui
        .selectable_label(run.show_comments, comments_label)
        .on_hover_text(crate::view::tips::comments())
        .clicked()
    {
        out.push(Msg::RehearsalEdit(
            crate::rehearsal_edit::RehearsalEditMsg::ToggleComments,
        ));
    }
    if s.run.mode != RunMode::Exam
        && let Some(uid) = studyfrog_rehearsal::session::current_item(s)
        && ui
            .button("Edit (Ctrl+E)")
            .on_hover_text("Opens this for editing (Ctrl+E).")
            .clicked()
    {
        out.push(Msg::RehearsalEdit(
            crate::rehearsal_edit::RehearsalEditMsg::Open(uid),
        ));
    }
    if s.run.state == RunState::Running
        && ui
            .button("Pause (P)")
            .on_hover_text("Pauses the timer and hides the question (P).")
            .clicked()
    {
        act(out, Action::Pause);
    }
    let stop = if s.run.mode == RunMode::Exam {
        "Stop exam (not graded)"
    } else {
        "Stop (Esc)"
    };
    if ui
        .button(stop)
        .on_hover_text("Ends the run now and keeps what you answered.")
        .clicked()
    {
        act(out, Action::Abort);
    }
}
