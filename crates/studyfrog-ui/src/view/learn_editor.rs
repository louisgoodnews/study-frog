//! The learn-mode editor's form (STORY-0198): the stored modes, the nodes and wires of the open one, the points
//! that stop it being saved, and the save. Every control sends a message; nothing here changes the editor.

use eframe::egui::{self, ComboBox, RichText};
use studyfrog_service::learn_mode_store::kind_name;

use crate::learn_editor::{KINDS, LearnEditor, LearnMsg, WHENS};
use crate::modal::ModalMsg;
use crate::msg::Msg;
use crate::theme::palette_of;

pub mod report;

fn send(out: &mut Vec<Msg>, m: LearnMsg) {
    out.push(Msg::Learn(m));
}

/// The learn-mode dialog's contents.
pub fn draw(e: &LearnEditor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    // The form is taller than a small window: it scrolls inside the dialog, which stays within the window.
    let max = (ui.ctx().content_rect().height() * 0.85).max(240.0);
    ui.set_height(max);
    egui::ScrollArea::vertical()
        .max_height(max)
        .auto_shrink([false, true])
        .show(ui, |ui| body(e, ui, out));
}

fn body(e: &LearnEditor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal(|ui| {
        ui.heading("Learn modes");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .button("Close")
                .on_hover_text("Closes the learn-mode editor; a saved mode stays saved.")
                .clicked()
            {
                out.push(Msg::Modal(ModalMsg::Close));
            }
        });
    });
    ui.weak("A learn mode is a graph of steps. A practice-only run follows it and keeps its own log; it changes no schedule.");
    ui.add_space(6.0);
    stored(e, ui, out);
    ui.separator();
    name_row(e, ui, out);
    nodes(e, ui, out);
    wires(e, ui, out);
    ui.separator();
    points(e, ui);
    report::section(e, ui, out);
    ui.horizontal(|ui| {
        if ui
            .add_enabled(e.reasons.is_empty(), egui::Button::new("Save mode"))
            .on_hover_text(
                "Saves the mode into the vault. Disabled while the points below are open.",
            )
            .clicked()
        {
            send(out, LearnMsg::Save);
        }
        if ui
            .add_enabled(
                e.file.is_some() && e.reasons.is_empty(),
                egui::Button::new("Start a run"),
            )
            .on_hover_text(
                "Runs the saved copy of this mode as a practice-only run: it changes no schedule.",
            )
            .clicked()
        {
            send(out, LearnMsg::Start);
        }
        if let Some(n) = &e.notice {
            ui.label(n);
        }
    });
}

fn stored(e: &LearnEditor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.label(RichText::new("Stored modes").strong());
    ui.horizontal_wrapped(|ui| {
        if ui
            .button("New mode")
            .on_hover_text("Starts an empty mode. Save it to keep it.")
            .clicked()
        {
            send(out, LearnMsg::NewMode);
        }
        for file in &e.modes {
            let open = e.file.as_deref() == Some(file.as_str());
            if ui.selectable_label(open, file).clicked() {
                send(out, LearnMsg::Pick(file.clone()));
            }
        }
    });
    if e.modes.is_empty() {
        ui.weak("None saved yet.");
    }
}

fn name_row(e: &LearnEditor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let mut name = e.graph.name.clone();
    ui.horizontal(|ui| {
        ui.label("Name");
        if ui
            .add(egui::TextEdit::singleline(&mut name).desired_width(260.0))
            .changed()
        {
            send(out, LearnMsg::Name(name.clone()));
        }
    });
}

fn nodes(e: &LearnEditor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.add_space(4.0);
    ui.label(RichText::new("Nodes").strong());
    for n in &e.graph.nodes {
        ui.horizontal(|ui| {
            ui.monospace(&n.id);
            ui.label(format!("{} · {}", kind_name(n.kind), n.label));
            if ui
                .small_button("Remove")
                .on_hover_text("Removes this step, or this wire, from the mode.")
                .clicked()
            {
                send(out, LearnMsg::RemoveNode(n.id.clone()));
            }
        });
    }
    let mut kind = e.kind;
    let mut label = e.label.clone();
    ui.horizontal(|ui| {
        ComboBox::from_id_salt("learn-node-kind")
            .selected_text(kind_name(kind))
            .show_ui(ui, |ui| {
                for k in KINDS {
                    ui.selectable_value(&mut kind, k, kind_name(k));
                }
            });
        if kind != e.kind {
            send(out, LearnMsg::Kind(kind));
        }
        if ui
            .add(
                egui::TextEdit::singleline(&mut label)
                    .hint_text("label")
                    .desired_width(180.0),
            )
            .changed()
        {
            send(out, LearnMsg::Label(label));
        }
        if ui
            .button("Add node")
            .on_hover_text(
                "Adds a step of the chosen kind, labelled with the text typed (or its kind).",
            )
            .clicked()
        {
            send(out, LearnMsg::AddNode);
        }
    });
}

fn wires(e: &LearnEditor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.add_space(4.0);
    ui.label(RichText::new("Wires").strong());
    for (i, w) in e.graph.edges.iter().enumerate() {
        ui.horizontal(|ui| {
            let when = w
                .when
                .as_deref()
                .map(|c| format!(" when {c}"))
                .unwrap_or_default();
            ui.monospace(format!("{} -> {}{when}", w.from, w.to));
            if ui
                .small_button("Remove")
                .on_hover_text("Removes this step, or this wire, from the mode.")
                .clicked()
            {
                send(out, LearnMsg::RemoveWire(i));
            }
        });
    }
    let ids: Vec<String> = e.graph.nodes.iter().map(|n| n.id.clone()).collect();
    let mut from = e.from.clone();
    let mut to = e.to.clone();
    let mut when = e.when.clone();
    ui.horizontal(|ui| {
        pick_node(ui, "learn-from", "from", &ids, &mut from);
        pick_node(ui, "learn-to", "to", &ids, &mut to);
        ComboBox::from_id_salt("learn-when")
            .selected_text(if when.is_empty() {
                "plain"
            } else {
                when.as_str()
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut when, String::new(), "plain");
                for c in WHENS {
                    ui.selectable_value(&mut when, c.to_owned(), c);
                }
            });
        if ui.button("Add wire").on_hover_text("Adds a wire from the first step to the second, with the condition chosen (plain if none).").clicked() {
            send(out, LearnMsg::AddWire);
        }
    });
    if from != e.from {
        send(out, LearnMsg::From(from));
    }
    if to != e.to {
        send(out, LearnMsg::To(to));
    }
    if when != e.when {
        send(out, LearnMsg::When(when));
    }
}

fn pick_node(ui: &mut egui::Ui, salt: &str, hint: &str, ids: &[String], value: &mut String) {
    ComboBox::from_id_salt(salt)
        .selected_text(if value.is_empty() {
            hint
        } else {
            value.as_str()
        })
        .show_ui(ui, |ui| {
            for id in ids {
                ui.selectable_value(value, id.clone(), id);
            }
        });
}

fn points(e: &LearnEditor, ui: &mut egui::Ui) {
    let p = palette_of(ui);
    if e.reasons.is_empty() {
        ui.label(RichText::new("Ready to save: the mode can run.").color(p.success));
        return;
    }
    ui.label(
        RichText::new("Before it can be saved:")
            .color(p.warning)
            .strong(),
    );
    for r in &e.reasons {
        ui.label(format!("• {r}"));
    }
}
