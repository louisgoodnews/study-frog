//! The exam part of the assistant screen: choose stacks and a length, answer, record the verdicts.

use eframe::egui::{self, RichText};
use studyfrog_agent::exam::Verdict;
use studyfrog_agent::provider::Role;

use crate::agent_types::AgentMsg;
use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::agent_wiki::{busy, scope, send};
use crate::view::kit;
use crate::view::widgets::text_field;

fn verdict_word(v: Verdict) -> &'static str {
    match v {
        Verdict::Correct => "correct",
        Verdict::Partial => "partly right",
        Verdict::Incorrect => "not right",
    }
}

/// Be examined: choose stacks and a length, answer, then record the verdicts.
pub fn exam(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let a = &state.agent;
    let weak = palette_of(ui).text_weak;
    scope(state, ui, out);
    if let Some(t) = text_field(ui, "Number of questions", &a.exam_count, "5", 1) {
        send(out, AgentMsg::ExamCount(t));
    }
    ui.horizontal(|ui| {
        if ui
            .add_enabled(!a.busy, egui::Button::new("Start exam"))
            .clicked()
        {
            send(out, AgentMsg::StartExam);
        }
        busy(a, ui);
    });
    for m in a.exam.chat.iter().skip(1) {
        kit::card(ui, |ui| {
            ui.set_width(ui.available_width());
            let who = if m.role == Role::User {
                "You"
            } else {
                "Examiner"
            };
            ui.label(RichText::new(who).small().color(weak));
            ui.add(egui::Label::new(&m.content).wrap());
        });
    }
    if !a.exam.items.is_empty() && a.exam.results.is_none() {
        if let Some(t) = text_field(ui, "Your answer", &a.exam.input, "", 3) {
            send(out, AgentMsg::ExamInput(t));
        }
        if ui
            .add_enabled(!a.busy, egui::Button::new("Send answer"))
            .clicked()
        {
            send(out, AgentMsg::ExamSend);
        }
    }
    if let Some(results) = &a.exam.results {
        kit::section(ui, "Result");
        for r in results {
            let title = a
                .exam
                .items
                .iter()
                .find(|i| i.uid == r.uid)
                .map_or(r.uid.as_str(), |i| i.prompt.as_str());
            ui.label(format!("{title}: {}", verdict_word(r.verdict)));
        }
        match a.exam.recorded {
            Some(n) => ui.label(format!("Recorded {n} reviews.")),
            None => {
                ui.label(
                    RichText::new("Recording counts these as reviews and updates your schedule.")
                        .color(weak),
                );
                if kit::primary_button(ui, "Record as reviews").clicked() {
                    send(out, AgentMsg::RecordExam);
                }
                ui.label("")
            }
        };
    }
}
