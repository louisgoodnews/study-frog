//! The option list of a choice question in the item editor (STORY-0224). It is a view of the Markdown prompt: each
//! change is written back through the editor's field save, and the answer type through the front matter. A change
//! that would leave the options disagreeing with the answer type is refused, and the editor says so.

use eframe::egui::{self, RichText, TextEdit};

use crate::editor::Editor;
use crate::msg::{EditAction, Msg, MsgEditor};
use crate::theme::palette_of;
use crate::view::choice_body::{
    AnswerType, Row, agrees, answer_type_label, block_of, open_text_body, render, rows_of,
};

/// One change the user made to the list this frame.
enum Act {
    Text(usize, String),
    Mark(usize),
    Remove(usize),
    Add,
}

fn edit(out: &mut Vec<Msg>, action: EditAction) {
    out.push(Msg::Editor(MsgEditor::Edit(action)));
}

/// The egui memory key for one of the panel's states, per item.
fn key(e: &Editor, what: &str) -> egui::Id {
    egui::Id::unique(("choice-editor", e.path.as_str(), what))
}

/// What a refused change says, for the answer type it would break.
fn refusal(t: AnswerType) -> &'static str {
    match t {
        AnswerType::Open => "Open text has no options. Choose the answer type first.",
        AnswerType::Single => "Single choice needs exactly one correct option. Nothing was saved.",
        AnswerType::Multiple => {
            "Multiple select needs two or more correct options. Nothing was saved."
        }
    }
}

/// The answer type buttons: switching to open text asks first, because it removes the option lines.
fn type_switch(e: &Editor, rows: &[Row], prompt: &str, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let stated = e.form.answer_type;
    let confirm_id = key(e, "confirm-open");
    let asking = ui.data(|d| d.get_temp::<bool>(confirm_id)).unwrap_or(false);
    ui.horizontal_wrapped(|ui| {
        for kind in [AnswerType::Single, AnswerType::Multiple, AnswerType::Open] {
            let hover = if kind == AnswerType::Open {
                "Removes the option lines and makes this an open-text question, after a confirmation."
            } else if agrees(kind, rows) {
                "Changes the answer type. The options already agree with it."
            } else {
                "The options do not agree with this type yet; the change is refused until they do."
            };
            if ui
                .selectable_label(stated == kind, answer_type_label(kind))
                .on_hover_text(hover)
                .clicked()
            {
                if kind == AnswerType::Open {
                    ui.data_mut(|d| d.insert_temp(confirm_id, true));
                } else if agrees(kind, rows) {
                    edit(out, EditAction::AnswerType(kind));
                } else {
                    ui.data_mut(|d| d.insert_temp(key(e, "notice"), refusal(kind).to_owned()));
                }
            }
        }
    });
    if asking {
        ui.add_space(4.0);
        ui.label("Removing the option lines makes this an open-text question. Continue?");
        ui.horizontal(|ui| {
            if ui
                .button("Make open text")
                .on_hover_text("Removes the option lines and keeps the prompt.")
                .clicked()
            {
                studyfrog_log::log_info!("ui.choice", "answer type changed to open text");
                edit(out, EditAction::Field(0, open_text_body(prompt)));
                edit(out, EditAction::AnswerType(AnswerType::Open));
                ui.data_mut(|d| d.insert_temp(confirm_id, false));
            }
            if ui
                .button("Keep options")
                .on_hover_text("Keeps the options as they are.")
                .clicked()
            {
                ui.data_mut(|d| d.insert_temp(confirm_id, false));
            }
        });
    }
}

/// One option: its text, the correct/wrong switch and the remove button.
fn option_row(ui: &mut egui::Ui, i: usize, row: &Row, acts: &mut Vec<Act>) {
    let p = palette_of(ui);
    ui.horizontal(|ui| {
        let mut text = row.text.clone();
        let field = TextEdit::singleline(&mut text)
            .hint_text("Option text")
            .desired_width(220.0);
        if ui.add(field).changed() {
            acts.push(Act::Text(i, text));
        }
        let (label, color, hover) = if row.correct {
            (
                "Correct",
                p.success,
                "Counts as a correct answer. Click to mark it wrong.",
            )
        } else {
            (
                "Wrong",
                p.text_weak,
                "Counts as a wrong answer. Click to mark it correct.",
            )
        };
        if ui
            .button(RichText::new(label).color(color))
            .on_hover_text(hover)
            .clicked()
        {
            acts.push(Act::Mark(i));
        }
        if ui
            .small_button("Remove")
            .on_hover_text("Removes this option from the item.")
            .clicked()
        {
            acts.push(Act::Remove(i));
        }
    });
}

/// Applies the list's changes; a change that breaks the answer type is refused, a text change always applies.
fn apply(
    e: &Editor,
    block_prompt: &str,
    rows: &[Row],
    acts: Vec<Act>,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    let mut next = rows.to_vec();
    let mut counts_changed = false;
    for act in acts {
        match act {
            Act::Text(i, text) => {
                if let Some(r) = next.get_mut(i) {
                    r.text = text;
                }
            }
            Act::Mark(i) => {
                if let Some(r) = next.get_mut(i) {
                    r.correct = !r.correct;
                    counts_changed = true;
                }
            }
            Act::Remove(i) => {
                if i < next.len() {
                    next.remove(i);
                    counts_changed = true;
                    studyfrog_log::log_info!("ui.choice", "option removed, {} left", next.len());
                }
            }
            Act::Add => {
                next.push(Row {
                    text: String::new(),
                    correct: false,
                });
                counts_changed = true;
                studyfrog_log::log_info!("ui.choice", "option added, {} now", next.len());
            }
        }
    }
    if counts_changed && !agrees(e.form.answer_type, &next) {
        studyfrog_log::log_info!(
            "ui.choice",
            "save refused: options do not agree with the answer type"
        );
        ui.data_mut(|d| d.insert_temp(key(e, "notice"), refusal(e.form.answer_type).to_owned()));
        return;
    }
    ui.data_mut(|d| d.remove::<String>(key(e, "notice")));
    edit(out, EditAction::Field(0, render(block_prompt, &next)));
}

/// Draws the option list under the prompt when the prompt holds a choice.
pub fn panel(e: &Editor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(first) = e.form.fields.first() else {
        return;
    };
    let Some(block) = block_of(&first.text) else {
        return;
    };
    let rows = rows_of(&block);
    studyfrog_debug::check(
        "ui.choice",
        rows.len() >= 2,
        "choice body has at least two options",
    );
    studyfrog_debug::dump("ui.choice", "options", &rows);
    let stated = e.form.answer_type;
    let p = palette_of(ui);

    ui.add_space(6.0);
    ui.label(RichText::new("Options").color(p.text_weak));
    type_switch(e, &rows, &first.text, ui, out);
    if stated != AnswerType::Open && !agrees(stated, &rows) {
        let hint = match stated {
            AnswerType::Single => "exactly one option must be correct",
            _ => "two or more options must be correct",
        };
        ui.colored_label(
            p.warning,
            format!("The options do not match the answer type: {hint}."),
        );
    }
    if let Some(notice) = ui.data(|d| d.get_temp::<String>(key(e, "notice"))) {
        ui.colored_label(p.danger, notice);
    }

    let mut acts = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        option_row(ui, i, row, &mut acts);
    }
    if ui
        .button("Add option")
        .on_hover_text("Adds an empty wrong option at the end.")
        .clicked()
    {
        acts.push(Act::Add);
    }
    if !acts.is_empty() {
        apply(e, &block.prompt, &rows, acts, ui, out);
    }
}
