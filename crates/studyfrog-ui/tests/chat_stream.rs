//! Integration tests for real-time chat stream state and UI renderer.

use eframe::egui;
use studyfrog_ui::theme::palette;
use studyfrog_ui::view::agent_chat_stream::{
    StreamChunk, apply_stream_chunk, clear_chat_stream, is_stream_active, new_chat_stream_state,
    render_chat_stream, stream_text,
};

#[test]
fn test_chat_stream_initial_and_thinking() {
    let mut state = new_chat_stream_state();
    assert!(!state.is_streaming);
    assert!(!state.is_thinking);
    assert!(state.buffer.is_empty());
    assert!(state.error.is_none());
    assert!(!is_stream_active(&state));

    apply_stream_chunk(
        &mut state,
        StreamChunk::Thinking("Analyzing vault notes...".to_string()),
    );
    assert!(state.is_streaming);
    assert!(state.is_thinking);
    assert!(is_stream_active(&state));
    assert_eq!(stream_text(&state), "");
}

#[test]
fn test_chat_stream_tokens_and_completion() {
    let mut state = new_chat_stream_state();
    apply_stream_chunk(
        &mut state,
        StreamChunk::Thinking("Analyzing...".to_string()),
    );
    apply_stream_chunk(&mut state, StreamChunk::Token("Hello".to_string()));
    assert!(state.is_streaming);
    assert!(!state.is_thinking);
    assert_eq!(stream_text(&state), "Hello");

    apply_stream_chunk(&mut state, StreamChunk::Token(" world!".to_string()));
    assert_eq!(stream_text(&state), "Hello world!");

    apply_stream_chunk(&mut state, StreamChunk::Complete(String::new()));
    assert!(!state.is_streaming);
    assert!(!state.is_thinking);
    assert!(!is_stream_active(&state));
    assert_eq!(stream_text(&state), "Hello world!");
}

#[test]
fn test_chat_stream_error_handling() {
    let mut state = new_chat_stream_state();
    apply_stream_chunk(
        &mut state,
        StreamChunk::Thinking("Connecting...".to_string()),
    );
    assert!(state.is_streaming);

    apply_stream_chunk(
        &mut state,
        StreamChunk::Error("Server connection timeout".to_string()),
    );
    assert!(!state.is_streaming);
    assert!(!state.is_thinking);
    assert_eq!(state.error, Some("Server connection timeout".to_string()));
}

#[test]
fn test_chat_stream_complete_with_payload() {
    let mut state = new_chat_stream_state();
    apply_stream_chunk(
        &mut state,
        StreamChunk::Complete("Immediate full response".to_string()),
    );
    assert!(!state.is_streaming);
    assert_eq!(stream_text(&state), "Immediate full response");
}

#[test]
fn test_clear_chat_stream() {
    let mut state = new_chat_stream_state();
    apply_stream_chunk(&mut state, StreamChunk::Token("Some tokens".to_string()));
    apply_stream_chunk(&mut state, StreamChunk::Error("Some error".to_string()));

    clear_chat_stream(&mut state);
    assert!(!state.is_streaming);
    assert!(!state.is_thinking);
    assert!(state.buffer.is_empty());
    assert!(state.error.is_none());
}

#[test]
fn test_render_chat_stream_thinking() {
    let mut state = new_chat_stream_state();
    apply_stream_chunk(
        &mut state,
        StreamChunk::Thinking("Analyzing vault...".to_string()),
    );

    let pal = palette(true);
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        render_chat_stream(ui, &state, &pal);
    });
    output.textures_delta.clear();
}

#[test]
fn test_render_chat_stream_generating_tokens() {
    let mut state = new_chat_stream_state();
    apply_stream_chunk(
        &mut state,
        StreamChunk::Token("Evaluating memory intervals...".to_string()),
    );

    let pal = palette(false);
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        render_chat_stream(ui, &state, &pal);
    });
    output.textures_delta.clear();
}

#[test]
fn test_render_chat_stream_completed_and_error() {
    let mut state = new_chat_stream_state();
    apply_stream_chunk(&mut state, StreamChunk::Token("Final answer".to_string()));
    apply_stream_chunk(&mut state, StreamChunk::Complete(String::new()));
    state.error = Some("Subsequent warning or notice".to_string());

    let pal = palette(true);
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        render_chat_stream(ui, &state, &pal);
    });
    output.textures_delta.clear();
}
