//! Model provider traits and streaming chunk primitives (ADR 0009, Section 4.5).
//!
//! Complies strictly with Invariant P1 (zero inherent impls) and P6 (<= 250 LOC).

use std::sync::mpsc::Sender;

/// Stream chunk delivered during token generation across threads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamChunk {
    /// Prefill or reasoning phase text (e.g. "Thinking / Analyzing vault...").
    Thinking(String),
    /// Emitted generation token.
    Token(String),
    /// Generation complete indicator with final accumulated text.
    Complete(String),
    /// Generation error message.
    Error(String),
}

/// Extracts text payload from a [`StreamChunk`], if applicable.
pub fn stream_chunk_text(chunk: &StreamChunk) -> Option<&str> {
    match chunk {
        StreamChunk::Thinking(s) | StreamChunk::Token(s) | StreamChunk::Complete(s) => {
            Some(s.as_str())
        }
        StreamChunk::Error(_) => None,
    }
}

/// Request parameters for model completion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionRequest {
    /// Prompt text.
    pub prompt: String,
    /// Maximum tokens to generate.
    pub max_tokens: u32,
    /// Sampling temperature (scaled by 100 to avoid floats in Eq, e.g. 70 = 0.7).
    pub temperature_cent: u32,
    /// Stop sequences.
    pub stop_sequences: Vec<String>,
}

/// Creates a new [`CompletionRequest`] with default parameters.
pub fn new_completion_request(prompt: String) -> CompletionRequest {
    CompletionRequest {
        prompt,
        max_tokens: 512,
        temperature_cent: 70,
        stop_sequences: Vec::new(),
    }
}

/// Non-streaming completion response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionResponse {
    /// Full generated text.
    pub text: String,
    /// Model name that produced the completion.
    pub model: String,
    /// Prompt tokens evaluated.
    pub prompt_tokens: u32,
    /// Completion tokens generated.
    pub completion_tokens: u32,
}

/// Common text completion provider trait (Section 4.5).
pub trait ModelProvider: Send + Sync {
    /// Identifies the model provider name.
    fn model_name(&self) -> &str;

    /// Executes single-shot text completion.
    fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, String>;

    /// Executes streaming text completion, sending tokens asynchronously over a channel.
    fn complete_streaming(
        &self,
        req: &CompletionRequest,
        sender: Sender<StreamChunk>,
    ) -> Result<(), String>;
}
