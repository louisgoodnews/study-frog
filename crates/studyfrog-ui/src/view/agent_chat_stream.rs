//! Real-time Server-Sent Events token streaming renderer for chat views (Section 4.5).
//!
//! Complies strictly with Invariant P1 (zero inherent impls) and P6 (<= 250 LOC).

use eframe::egui::{self, RichText};

pub use studyfrog_agent::model_provider::StreamChunk;

use crate::theme::{Palette, tint};

/// Streaming buffer and animation state for the chat assistant view.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChatStreamState {
    /// Whether token streaming is currently active.
    pub is_streaming: bool,
    /// Whether the model is currently in prefill or vault retrieval reasoning.
    pub is_thinking: bool,
    /// Accumulated text buffer of generated tokens.
    pub buffer: String,
    /// Error message if token generation failed.
    pub error: Option<String>,
}

/// Creates a new, blank [`ChatStreamState`].
pub fn new_chat_stream_state() -> ChatStreamState {
    ChatStreamState {
        is_streaming: false,
        is_thinking: false,
        buffer: String::new(),
        error: None,
    }
}

/// Resets a [`ChatStreamState`] to empty and inactive.
pub fn clear_chat_stream(state: &mut ChatStreamState) {
    state.is_streaming = false;
    state.is_thinking = false;
    state.buffer.clear();
    state.error = None;
}

/// Returns whether the stream is actively generating or reasoning.
pub fn is_stream_active(state: &ChatStreamState) -> bool {
    state.is_streaming || state.is_thinking
}

/// Returns the currently accumulated text buffer.
pub fn stream_text(state: &ChatStreamState) -> &str {
    &state.buffer
}

/// Updates the streaming buffer state upon receiving a new [`StreamChunk`].
pub fn apply_stream_chunk(state: &mut ChatStreamState, chunk: StreamChunk) {
    match chunk {
        StreamChunk::Thinking(_msg) => {
            state.is_streaming = true;
            state.is_thinking = true;
            state.error = None;
        }
        StreamChunk::Token(token) => {
            state.is_streaming = true;
            // The first token chunk seamlessly terminates the prefill thinking indicator
            state.is_thinking = false;
            state.error = None;
            state.buffer.push_str(&token);
        }
        StreamChunk::Complete(final_text) => {
            state.is_streaming = false;
            state.is_thinking = false;
            if !final_text.is_empty() && state.buffer.is_empty() {
                state.buffer = final_text;
            }
        }
        StreamChunk::Error(err) => {
            state.is_streaming = false;
            state.is_thinking = false;
            state.error = Some(err);
        }
    }
}

/// Renders the real-time chat stream, handling pulsing prefill feedback and live tokens.
pub fn render_chat_stream(ui: &mut egui::Ui, state: &ChatStreamState, p: &Palette) {
    let now = ui.input(|i| i.time);

    // 1. Render accumulated response card if tokens exist
    if !state.buffer.is_empty() {
        egui::Frame::new()
            .fill(p.surface)
            .stroke(egui::Stroke::new(1.0, p.border))
            .corner_radius(6)
            .inner_margin(8)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    RichText::new("Assistant")
                        .small()
                        .strong()
                        .color(p.text_weak),
                );
                ui.add_space(2.0);

                let display_text = if state.is_streaming && !state.is_thinking {
                    let cursor_active = (now * 2.5).sin() > 0.0;
                    if cursor_active {
                        format!("{}_", state.buffer)
                    } else {
                        format!("{} ", state.buffer)
                    }
                } else {
                    state.buffer.clone()
                };

                ui.add(egui::Label::new(RichText::new(display_text).color(p.text)).wrap());
            });

        ui.add_space(6.0);
    }

    // 2. Render pulsing prefill / thinking indicator during prefill phase
    if state.is_thinking {
        let pulse = ((now * 3.5).sin().abs() as f32).clamp(0.25, 1.0);
        let pulse_alpha = (70.0 + pulse * 185.0) as u8;
        let pulse_color = tint(p.accent, pulse_alpha);

        ui.horizontal(|ui| {
            ui.spinner();
            ui.label(
                RichText::new("Thinking / Analyzing vault...")
                    .italics()
                    .color(pulse_color),
            );
        });
        ui.add_space(4.0);

        // Keep egui loop animating smoothly during thinking
        ui.ctx().request_repaint();
    } else if state.is_streaming {
        // Continuous repaint for smooth token cursor blinking
        ui.ctx().request_repaint();
    }

    // 3. Render streaming error if encountered
    if let Some(err) = &state.error {
        egui::Frame::new()
            .fill(tint(p.danger, 25))
            .stroke(egui::Stroke::new(1.0, p.danger))
            .corner_radius(6)
            .inner_margin(8)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Error:").strong().color(p.danger));
                    ui.label(RichText::new(err).color(p.danger));
                });
            });
        ui.add_space(6.0);
    }
}
