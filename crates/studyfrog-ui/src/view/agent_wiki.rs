//! The two working parts of the assistant screen: proposals for the vault and exams.

use eframe::egui::{self, RichText};
use studyfrog_agent::provider::{host_of, is_local};
use studyfrog_agent::wiki::Change;

use crate::agent_types::{AgentMsg, AgentState, selected_provider};
use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::kit;
use crate::view::widgets::text_field;

pub(super) fn send(out: &mut Vec<Msg>, m: AgentMsg) {
    out.push(Msg::Agent(m));
}

/// Which model is in use, and the stacks it may read.
pub(super) fn scope(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let a = &state.agent;
    let weak = palette_of(ui).text_weak;
    match selected_provider(a) {
        Some(p) => ui.label(RichText::new(format!("Using {} ({})", p.name, p.model)).color(weak)),
        None => ui.colored_label(
            ui.visuals().error_fg_color,
            "No connection chosen: add one on the Connections tab.",
        ),
    };
    ui.label("Stacks the assistant may read");
    if let Some(v) = &state.vault {
        for s in &v.overview.stacks {
            let mut on = a.folders.contains(&s.folder);
            if ui
                .checkbox(&mut on, format!("Stack: {}", s.title))
                .changed()
            {
                send(out, AgentMsg::Folder(s.folder.clone(), on));
            }
        }
    }
}

pub(super) fn busy(a: &AgentState, ui: &mut egui::Ui) {
    if a.busy {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label("Waiting for the model…");
        });
    }
}

fn describe(c: &Change) -> (String, String) {
    match c {
        Change::CreateNote { stack, title, body } => {
            (format!("New note \"{title}\" in {stack}"), body.clone())
        }
        Change::CreateCard { stack, front, back } => (
            format!("New card in {stack}: {front}"),
            format!("Front: {front}\nBack: {back}"),
        ),
        Change::CreateQuestion {
            stack,
            prompt,
            reference,
        } => (
            format!("New question in {stack}: {prompt}"),
            format!("Prompt: {prompt}\nReference answer: {reference}"),
        ),
        Change::UpdateItem { uid, fields } => (
            format!("Change item {uid}"),
            fields
                .iter()
                .map(|(k, v)| format!("{k}: {v}"))
                .collect::<Vec<_>>()
                .join("\n"),
        ),
    }
}

/// Work the vault: choose stacks, say what to do, review the proposal.
pub fn wiki(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let a = &state.agent;
    scope(state, ui, out);
    if let Some(t) = text_field(ui, "What should the assistant do?", &a.instruction, "", 3) {
        send(out, AgentMsg::Instruction(t));
    }
    ui.horizontal(|ui| {
        if ui
            .add_enabled(!a.busy, egui::Button::new("Ask the assistant"))
            .clicked()
        {
            send(out, AgentMsg::PrepareWiki);
        }
        busy(a, ui);
    });
    if let Some(p) = &a.pending {
        pending(state, p.items.len(), (p.chars, p.left_out), ui, out);
    }
    if let Some(r) = &a.reviewing {
        review(r, ui, out);
    }
    if let Some(done) = &a.applied {
        ui.label(format!(
            "Applied: {} created, {} updated.",
            done.created, done.updated
        ));
        for f in &done.failed {
            ui.colored_label(ui.visuals().error_fg_color, f);
        }
    }
}

fn pending(
    state: &State,
    items: usize,
    (chars, left_out): (usize, usize),
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    let host = selected_provider(&state.agent).map_or_else(String::new, |p| {
        if is_local(&p) {
            "this computer".to_owned()
        } else {
            host_of(&p.base_url).to_owned()
        }
    });
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(
            RichText::new(format!(
                "This sends the text of {items} items ({chars} characters) to {host}."
            ))
            .strong(),
        );
        if left_out > 0 {
            ui.label(format!(
                "{left_out} more items do not fit and are left out."
            ));
        }
        ui.horizontal(|ui| {
            if kit::primary_button(ui, "Send").clicked() {
                send(out, AgentMsg::SendWiki);
            }
            if ui.button("Cancel").clicked() {
                send(out, AgentMsg::Discard);
            }
        });
    });
}

fn review(r: &crate::agent_types::Reviewing, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let weak = palette_of(ui).text_weak;
    kit::section(ui, "Proposal");
    if !r.proposal.summary.is_empty() {
        ui.label(&r.proposal.summary);
    }
    for why in &r.proposal.rejected {
        ui.label(RichText::new(why).color(weak));
    }
    for (i, c) in r.proposal.changes.iter().enumerate() {
        let (title, text) = describe(c);
        kit::card(ui, |ui| {
            ui.set_width(ui.available_width());
            let mut on = r.accepted.get(i).copied().unwrap_or(false);
            if ui
                .checkbox(&mut on, format!("Accept change {}: {title}", i + 1))
                .changed()
            {
                send(out, AgentMsg::Accept(i, on));
            }
            ui.add(
                egui::Label::new(
                    RichText::new(text.chars().take(600).collect::<String>())
                        .color(weak)
                        .small(),
                )
                .wrap(),
            );
        });
    }
    ui.horizontal(|ui| {
        if kit::primary_button(ui, "Apply accepted changes").clicked() {
            send(out, AgentMsg::ApplyProposal);
        }
        if ui.button("Discard proposal").clicked() {
            send(out, AgentMsg::Discard);
        }
    });
}
