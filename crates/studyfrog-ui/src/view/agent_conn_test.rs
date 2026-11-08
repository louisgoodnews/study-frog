//! The "Test connection" button of one connection and the outcome of its last test.

use eframe::egui::{self, RichText};
use studyfrog_agent::provider::{Provider, is_local};

use crate::agent_conn_test::{TestMsg, result_of};
use crate::agent_types::{AgentMsg, AgentState};
use crate::msg::Msg;
use crate::theme::palette_of;

/// The button; a remote connection must be allowed first, as for every request.
pub fn button(a: &AgentState, p: &Provider, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let running = a.conn_tests.running.contains(&p.name);
    let may = is_local(p) || p.allowed;
    let label = if running {
        "Testing...".to_owned()
    } else {
        format!("Test connection to {}", p.name)
    };
    let response = ui
        .add_enabled(may && !running, egui::Button::new(label).small())
        .on_disabled_hover_text("Allow sending to this provider first; the test is a real request.")
        .on_hover_text(
            "Send one tiny request (no text from your vault) and show whether it answers.",
        );
    if response.clicked() {
        out.push(Msg::Agent(AgentMsg::Test(TestMsg::Start(p.name.clone()))));
    }
}

/// The outcome of the last test, if any.
pub fn outcome(a: &AgentState, p: &Provider, ui: &mut egui::Ui) {
    let pal = palette_of(ui);
    match result_of(&a.conn_tests, &p.name) {
        Some(Ok(text)) => {
            ui.label(RichText::new(format!("Works: {text}")).color(pal.success));
        }
        Some(Err(e)) => {
            ui.label(RichText::new(format!("Connection test failed: {e}")).color(pal.danger));
        }
        None => {}
    }
}
