//! The focus timer in the top bar (STORY-0112): a quiet "Focus" button while idle, the phase and time left
//! while it runs; its menu starts, pauses, resets, skips the break and picks the preset and the stack. A card
//! offers the break and a diary entry when a work block ends.

use eframe::egui::{self, RichText};

use crate::focus_timer::{self as ft, PRESETS, Phase, Preset};
use crate::focus_ui::{FocusMsg, msg};
use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::kit;

fn label(state: &State) -> String {
    let f = &state.recall.focus;
    if ft::is_idle(f) {
        return "Focus".to_owned();
    }
    let word = match (f.phase, ft::is_running(f)) {
        (Phase::Work, true) => "Focus",
        (Phase::Work, false) => "Paused",
        (_, _) => "Break",
    };
    format!("{word} {}", ft::clock(ft::remaining(f, state.now)))
}

/// The pill's hover text: what the timer is for.
fn tip(state: &State) -> String {
    let f = &state.recall.focus;
    match f.stack.as_deref().and_then(|s| stack_title(state, s)) {
        Some(t) => format!("Focus timer · {t}"),
        None => "Focus timer".to_owned(),
    }
}

fn stack_title<'a>(state: &'a State, folder: &str) -> Option<&'a str> {
    let v = state.vault.as_ref()?;
    v.overview
        .stacks
        .iter()
        .find(|s| s.folder == folder)
        .map(|s| s.title.as_str())
}

/// The button in the top bar and its menu.
pub fn pill(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let f = &state.recall.focus;
    if ft::is_running(f) {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(500));
    }
    let r = if ft::is_idle(f) {
        crate::view::icons::icon_button(ui, crate::view::icons::Glyph::Timer, "Focus timer", false)
    } else {
        let text = RichText::new(label(state)).color(palette_of(ui).accent);
        ui.button(text)
    };
    egui::Popup::menu(&r)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| menu(state, ui, out));
    r.on_hover_text(tip(state));
}

fn controls(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let f = &state.recall.focus;
    ui.horizontal(|ui| {
        let (go, word) = if ft::is_running(f) {
            (FocusMsg::Pause, "Pause the timer")
        } else if ft::is_idle(f) {
            (FocusMsg::Start, "Start focusing")
        } else {
            (FocusMsg::Start, "Resume the timer")
        };
        if kit::primary_button(ui, word).clicked() {
            out.push(msg(go));
        }
        if f.phase != Phase::Work && ui.button("Skip the break").clicked() {
            out.push(msg(FocusMsg::SkipBreak));
        }
        if !ft::is_idle(f) && ui.button("Reset the timer").clicked() {
            out.push(msg(FocusMsg::Reset));
        }
    });
}

fn presets(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let p = state.recall.focus.preset;
    ui.horizontal(|ui| {
        for x in PRESETS {
            let name = format!("{} / {}", x.work, x.short);
            if ui.selectable_label(p == x, name).clicked() {
                out.push(msg(FocusMsg::Preset(x)));
            }
        }
    });
    ui.horizontal(|ui| {
        let (mut work, mut brk) = (p.work, p.short);
        ui.label("Custom: work");
        let a = ui.add(
            egui::DragValue::new(&mut work)
                .range(1..=600)
                .suffix(" min"),
        );
        ui.label("break");
        let b = ui.add(egui::DragValue::new(&mut brk).range(1..=120).suffix(" min"));
        if a.changed() || b.changed() {
            let long = brk.saturating_mul(3).min(240);
            out.push(msg(FocusMsg::Preset(Preset {
                work,
                short: brk,
                long,
            })));
        }
    });
}

fn stack_choice(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(v) = &state.vault else { return };
    let chosen = state.recall.focus.stack.as_deref();
    let shown = chosen
        .and_then(|s| stack_title(state, s))
        .unwrap_or("No stack");
    egui::ComboBox::from_label("Stack")
        .selected_text(shown)
        .show_ui(ui, |ui| {
            if ui.selectable_label(chosen.is_none(), "No stack").clicked() {
                out.push(msg(FocusMsg::Stack(None)));
            }
            for s in &v.overview.stacks {
                if ui
                    .selectable_label(chosen == Some(s.folder.as_str()), &s.title)
                    .clicked()
                {
                    out.push(msg(FocusMsg::Stack(Some(s.folder.clone()))));
                }
            }
        });
}

fn menu(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let f = &state.recall.focus;
    ui.set_min_width(300.0);
    ui.label(RichText::new("Work and breaks").strong());
    ui.weak(format!(
        "{} work block(s) done. A long break follows every {}th.",
        f.blocks,
        ft::LONG_EVERY
    ));
    controls(state, ui, out);
    ui.separator();
    presets(state, ui, out);
    stack_choice(state, ui, out);
    if let Some(r) = &f.recorded {
        ui.weak(r);
    }
}

/// The card shown when a work block has ended.
pub fn done_card(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let f = &state.recall.focus;
    let Some(done) = &f.done else { return };
    let brk = if f.phase == Phase::LongBreak {
        f.preset.long
    } else {
        f.preset.short
    };
    egui::Area::new(egui::Id::unique("focus-done"))
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-16.0, 56.0))
        .show(ui.ctx(), |ui| {
            kit::card(ui, |ui| {
                ui.set_max_width(320.0);
                ui.label(
                    RichText::new(format!("Focus session complete ({} min)", done.minutes))
                        .strong(),
                );
                ui.horizontal_wrapped(|ui| {
                    if kit::primary_button(ui, &format!("Start break ({brk} min)")).clicked() {
                        out.push(msg(FocusMsg::StartBreak));
                    }
                    if ui.button("Log in Diary…").clicked() {
                        out.push(msg(FocusMsg::LogDiary));
                    }
                    if ui.button("Dismiss the focus card").clicked() {
                        out.push(msg(FocusMsg::Dismiss));
                    }
                });
            });
        });
}
