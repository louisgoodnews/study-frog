//! Parts of the rehearsal card: the reveal and mark controls and the source link.
use super::*;

pub(super) fn mark_row(run: &RunView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let chosen = crate::review::current_mark(run);
    ui.horizontal(|ui| {
        ui.weak("This item is:");
        for (difficulty, label, _) in MARKS {
            if ui
                .selectable_label(chosen == Some(difficulty), label)
                .on_hover_text(crate::view::tips::mark(difficulty))
                .clicked()
            {
                act(out, Action::Mark(difficulty));
            }
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some(uid) = current_item(&run.session)
                && ui
                    .small_button("Edit")
                    .on_hover_text("Opens this for editing.")
                    .clicked()
            {
                out.push(Msg::RehearsalEdit(
                    crate::rehearsal_edit::RehearsalEditMsg::Open(uid),
                ));
            }
        });
    });
}

pub(super) fn source_link(content: &ItemContent, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if let Some(source) = &content.source {
        crate::view::pdf_split::source_link(source, ui, out);
    }
}

pub(super) fn reveal_controls(run: &RunView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let answering = is_item_typed(run);
    if answering {
        if let Some(text) = text_field(ui, "Your answer", &run.answer, "type your answer", 3) {
            out.push(Msg::AnswerEdited(text));
        }
        let ready = !run.answer.trim().is_empty();
        if ui
            .add_enabled_ui(ready, |ui| {
                kit::primary_button(ui, "Submit answer").on_hover_text("Checks your typed answer.")
            })
            .inner
            .clicked()
        {
            act(out, Action::SubmitAnswer(run.answer.clone()));
        }
        ui.add_space(4.0);
    }
    ui.horizontal(|ui| {
        let label = if answering {
            "Show answer without answering (Space)"
        } else {
            "Show answer (Space)"
        };
        let show = if answering {
            ui.button(label)
                .on_hover_text(crate::view::tips::show_answer())
        } else {
            kit::primary_button(ui, label).on_hover_text(crate::view::tips::show_answer())
        };
        if show
            .on_hover_text(crate::view::tips::show_answer())
            .clicked()
        {
            act(out, Action::Flip);
        }
        if ui
            .button("Skip (S)")
            .on_hover_text(crate::view::tips::skip())
            .clicked()
        {
            act(out, Action::Skip);
        }
    });
}
