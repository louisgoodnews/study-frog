//! The assistant screen's helper for AI clients that connect to StudyFrog: the settings block that
//! starts the Model Context Protocol server for the open vault.

use eframe::egui::{self, RichText};

use crate::mcp_stdio::mcp_config_snippet;
use crate::state::State;
use crate::theme::palette_of;

/// The program to start: the one running now, else `studyfrog` on the path.
fn program() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(str::to_owned))
        .unwrap_or_else(|| "studyfrog".to_owned())
}

/// A button that copies the settings block for the open vault; nothing without a vault.
pub fn snippet(state: &State, ui: &mut egui::Ui) {
    let Some(vault) = &state.vault else { return };
    ui.add_space(8.0);
    ui.label(
        RichText::new(
            "Other AI tools can read this vault through the Model Context Protocol (read-only). Paste this block into their settings.",
        )
        .color(palette_of(ui).text_weak),
    );
    if ui.button("Copy MCP server config").clicked() {
        ui.ctx()
            .copy_text(mcp_config_snippet(&program(), &vault.root));
    }
}
