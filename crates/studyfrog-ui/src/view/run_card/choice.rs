//! The options of a choice item on the rehearsal card (STORY-0222): radio buttons or checkboxes
//! before the answer, the marked options after it, and the keys that pick them.

use super::*;
use studyfrog_core::uid::Uid;
use studyfrog_md::choice::{ChoiceBlock, is_multiple};

use crate::msg::MsgRehearsal;
use crate::run_state::choice::{can_submit, picked_for};
use crate::run_update::choice::{ChoiceMsg, pending_block};

/// The prompt to draw: a pending choice item's prompt without its option list, else the item's own prompt.
pub(crate) fn prompt_or(
    run: &RunView,
    own: Vec<studyfrog_md::blocks::Block>,
) -> Vec<studyfrog_md::blocks::Block> {
    match pending_block(run) {
        Some((_, block)) => studyfrog_md::blocks::parse_blocks(&block.prompt),
        None => own,
    }
}

/// Number keys 1 to 9, option 1 to 9.
const NUMBER_KEYS: [Key; 9] = [
    Key::Num1,
    Key::Num2,
    Key::Num3,
    Key::Num4,
    Key::Num5,
    Key::Num6,
    Key::Num7,
    Key::Num8,
    Key::Num9,
];

/// The current item's choice block when the card shows its options: a typed item, awaiting the reveal
/// or already answered. An exam offers its options while the item awaits its submit, never after it:
/// an exam shows no answer until the end.
pub(crate) fn shown(run: &RunView) -> Option<(Uid, &ChoiceBlock)> {
    let exam = crate::exam_ui::is_exam(run);
    let offered = exam || is_item_typed(run);
    let open = awaiting_reveal(run) || (awaiting_rating(run) && !exam);
    if !offered || !open {
        return None;
    }
    pending_block(run)
}

/// Whether a choice item awaits its submit: the reveal is still open and the options are on screen.
pub(crate) fn pending(run: &RunView) -> bool {
    awaiting_reveal(run) && shown(run).is_some()
}

/// Number keys pick options and Enter submits, while a choice item awaits its submit.
pub(crate) fn keys(run: &RunView, ui: &egui::Ui, out: &mut Vec<Msg>) {
    let Some((uid, block)) = shown(run).filter(|_| pending(run)) else {
        return;
    };
    let len = block.options.len();
    let pressed = |k: Key| ui.input(|i| i.key_pressed(k));
    for (i, key) in NUMBER_KEYS.iter().enumerate().take(len) {
        if pressed(*key) {
            out.push(Msg::Rehearsal(MsgRehearsal::Choice(ChoiceMsg::Pick(i))));
        }
    }
    let ready = can_submit(&picked_for(&run.choice, uid, len));
    if ready && pressed(Key::Enter) {
        out.push(Msg::Rehearsal(MsgRehearsal::Choice(ChoiceMsg::Submit)));
    }
}

/// The options of the current choice item: picks before the answer, the marks after it.
pub(crate) fn options(
    run: &RunView,
    uid: Uid,
    block: &ChoiceBlock,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    let _timer = studyfrog_debug::timed("choice", "draw options");
    let picked = picked_for(&run.choice, uid, block.options.len());
    if awaiting_rating(run) {
        answered(block, &picked, ui);
    } else if crate::exam_ui::is_exam(run) {
        answering(
            block,
            &picked,
            (
                "Submit and lock answer",
                "Submits your choice and locks it.",
            ),
            ui,
            out,
        );
    } else {
        answering(
            block,
            &picked,
            (
                "Submit answer",
                "Checks the chosen options against the card (Enter).",
            ),
            ui,
            out,
        );
    }
}

/// Radio buttons for single choice, checkboxes for multiple select, then the Submit button (its label and
/// hover text given by the caller).
fn answering(
    block: &ChoiceBlock,
    picked: &[bool],
    (label, tip): (&str, &str),
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    let multiple = is_multiple(block);
    // The boxes and circles take a visible outline in the weak text colour, so they show on both themes.
    let p = crate::theme::palette_of(ui);
    ui.visuals_mut().widgets.inactive.bg_stroke = egui::Stroke::new(1.5, p.text_weak);
    ui.visuals_mut().widgets.hovered.bg_stroke = egui::Stroke::new(1.5, p.accent);
    for (i, option) in block.options.iter().enumerate() {
        let response = if multiple {
            let mut on = picked[i];
            let response = ui.checkbox(&mut on, option.text.as_str());
            if response.changed() {
                out.push(Msg::Rehearsal(MsgRehearsal::Choice(ChoiceMsg::Pick(i))));
            }
            response
        } else {
            let response = ui.radio(picked[i], option.text.as_str());
            if response.clicked() {
                out.push(Msg::Rehearsal(MsgRehearsal::Choice(ChoiceMsg::Pick(i))));
            }
            response
        };
        response.on_hover_text("Picks this option (1 to 9 on the keyboard).");
    }
    ui.add_space(6.0);
    let clicked = ui
        .add_enabled_ui(can_submit(picked), |ui| {
            kit::primary_button(ui, label).on_hover_text(tip)
        })
        .inner
        .clicked();
    if clicked {
        out.push(Msg::Rehearsal(MsgRehearsal::Choice(ChoiceMsg::Submit)));
    }
    ui.add_space(4.0);
}

/// The options after the answer: correct ones in the success colour, wrong picks in the danger colour.
fn answered(block: &ChoiceBlock, picked: &[bool], ui: &mut egui::Ui) {
    let p = palette_of(ui);
    for (i, option) in block.options.iter().enumerate() {
        let (color, note) = match (option.correct, picked[i]) {
            (true, true) => (p.success, " (correct, and you chose it)"),
            (true, false) => (p.success, " (correct)"),
            (false, true) => (p.danger, " (wrong, and you chose it)"),
            (false, false) => (p.text_weak, ""),
        };
        ui.label(RichText::new(format!("{}{note}", option.text)).color(color));
    }
}
