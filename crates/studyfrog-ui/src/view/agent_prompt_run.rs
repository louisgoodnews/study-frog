//! "Run a prompt with the assistant": choose a prompt and a file, see what will be sent, press Send, read the answer
//! and import it. Nothing is sent before Send, and not to a remote connection that was not allowed.

use eframe::egui::{self, ComboBox, RichText};
use studyfrog_service::prompts::{PROMPTS, prompt_of};

use crate::msg::Msg;
use crate::prompt_run::{PromptRun, PromptRunMsg, local};
use crate::prompts_ui::PromptsMsg;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::widgets::text_field;

fn send(out: &mut Vec<Msg>, m: PromptRunMsg) {
    out.push(Msg::Prompts(PromptsMsg::Run(m)));
}

fn chosen(run: &PromptRun) -> String {
    PROMPTS
        .iter()
        .find(|p| Some(p.id) == run.prompt)
        .map_or_else(|| "Choose a prompt".to_owned(), |p| prompt_of(p).title)
}

fn pickers(run: &PromptRun, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ComboBox::from_label("Prompt")
        .selected_text(chosen(run))
        .show_ui(ui, |ui| {
            for source in PROMPTS {
                let p = prompt_of(source);
                if ui
                    .selectable_label(run.prompt == Some(source.id), p.title)
                    .clicked()
                {
                    send(out, PromptRunMsg::Choose(source.id));
                }
            }
        });
    if let Some(text) = text_field(ui, "File to attach", &run.path, "/path/to/notes.txt", 1) {
        send(out, PromptRunMsg::Path(text));
    }
    ui.horizontal(|ui| {
        if ui.button("Choose file…").clicked() {
            send(out, PromptRunMsg::Browse);
        }
        if ui
            .add_enabled(!run.busy, egui::Button::new("Show what will be sent"))
            .clicked()
        {
            send(out, PromptRunMsg::Prepare);
        }
    });
}

fn review(
    state: &State,
    messages: &[studyfrog_agent::provider::Message],
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    let name = state
        .agent
        .selected
        .clone()
        .unwrap_or_else(|| "no connection".to_owned());
    let chars: usize = messages.iter().map(|m| m.content.chars().count()).sum();
    let place = if local(state) {
        "on this computer"
    } else {
        "to a provider over the network"
    };
    let cut = state.prompt_run.file.as_ref().is_some_and(|f| f.2);
    ui.label(format!(
        "This will be sent to {name} ({place}): {chars} characters{}.",
        if cut { ", the file cut to fit" } else { "" }
    ));
    egui::CollapsingHeader::new("Show the text that will be sent")
        .id_salt("prompt-outgoing")
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .max_height(220.0)
                .show(ui, |ui| {
                    for m in messages {
                        ui.add(
                            egui::Label::new(RichText::new(&m.content).monospace().size(12.0))
                                .wrap(),
                        );
                    }
                });
        });
    ui.horizontal(|ui| {
        if ui.button("Send").clicked() {
            send(out, PromptRunMsg::Send);
        }
        if ui.button("Don't send").clicked() {
            send(out, PromptRunMsg::Cancel);
        }
    });
}

fn answer(text: &str, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.label(RichText::new("The assistant's answer").strong());
    egui::ScrollArea::vertical()
        .id_salt("prompt-answer")
        .max_height(260.0)
        .show(ui, |ui| {
            ui.add(egui::Label::new(RichText::new(text).monospace().size(12.0)).wrap());
        });
    ui.horizontal(|ui| {
        if ui.button("Copy the answer").clicked() {
            ui.ctx().copy_text(text.to_owned());
        }
        if ui
            .button("Import this answer…")
            .on_hover_text("Shows a preview first; nothing is written yet.")
            .clicked()
        {
            send(out, PromptRunMsg::ImportReply);
        }
    });
}

/// The section under the list of prompts.
pub fn run(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let run = &state.prompt_run;
    ui.separator();
    ui.label(
        RichText::new("Run a prompt with the assistant")
            .size(18.0)
            .strong(),
    );
    ui.label(RichText::new("Uses the connection chosen on the Connections tab. The file is read here; you see exactly what would be sent and nothing is sent until you press Send.").color(palette_of(ui).text_weak));
    pickers(run, ui, out);
    if let Some(e) = &run.error {
        ui.colored_label(ui.visuals().error_fg_color, e);
    }
    if run.busy {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label("Working…");
        });
    }
    if let Some(messages) = &run.outgoing {
        review(state, messages, ui, out);
    } else if let Some(text) = &run.reply {
        answer(text, ui, out);
    }
}
