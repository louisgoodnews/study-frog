//! Prompt budgeting and vault context assembly for on-device SLM inference (TASK-0276).
use serde::{Deserialize, Serialize};

/// Budget constraints for assembling an on-device SLM prompt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptBudget {
    /// Total token ceiling.
    pub max_total_tokens: usize,
    /// System prompt token limit.
    pub max_system_tokens: usize,
    /// Conversation history token limit.
    pub max_history_tokens: usize,
    /// Context snippet token limit.
    pub max_context_tokens: usize,
}

/// Prompt assembled under budget constraints ready for SLM generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssembledPrompt {
    /// Truncated system prompt.
    pub system_prompt: String,
    /// Windowed history turns.
    pub conversation_history: Vec<(String, String)>,
    /// Selected context snippets.
    pub context_snippets: Vec<String>,
    /// Total estimated tokens.
    pub total_estimated_tokens: usize,
}

/// State indicator for the prefill / vault analysis phase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThinkingState {
    /// Whether analysis is active.
    pub is_analyzing: bool,
    /// Descriptive phase message.
    pub phase_message: String,
}

impl Default for PromptBudget {
    fn default() -> Self {
        default_prompt_budget()
    }
}

/// Returns the default 1024-token prompt budget (system: 150, history: 150, context: 768).
pub fn default_prompt_budget() -> PromptBudget {
    PromptBudget {
        max_total_tokens: 1024,
        max_system_tokens: 150,
        max_history_tokens: 150,
        max_context_tokens: 768,
    }
}

/// Estimates token count using standard heuristic (~4 characters per token).
pub fn estimate_token_count(text: &str) -> usize {
    text.chars().count().div_ceil(4)
}

/// Truncates `text` so its estimated token count does not exceed `max_tokens`.
pub fn truncate_to_tokens(text: &str, max_tokens: usize) -> String {
    if estimate_token_count(text) <= max_tokens {
        text.to_string()
    } else {
        text.chars().take(max_tokens * 4).collect()
    }
}

fn split_line_by_words(line: &str, max_tokens: usize, chunks: &mut Vec<String>) {
    let mut word_chunk = String::new();
    for word in line.split_whitespace() {
        let cand = if word_chunk.is_empty() {
            word.to_string()
        } else {
            format!("{word_chunk} {word}")
        };
        if estimate_token_count(&cand) > max_tokens && !word_chunk.is_empty() {
            chunks.push(word_chunk);
            word_chunk = word.to_string();
        } else {
            word_chunk = cand;
        }
    }
    if !word_chunk.is_empty() {
        chunks.push(word_chunk);
    }
}

fn split_large_paragraph(p: &str, max_tokens: usize, chunks: &mut Vec<String>) {
    let mut line_chunk = String::new();
    for line in p.lines().map(str::trim).filter(|l| !l.is_empty()) {
        if estimate_token_count(line) > max_tokens {
            if !line_chunk.is_empty() {
                chunks.push(line_chunk.clone());
                line_chunk.clear();
            }
            split_line_by_words(line, max_tokens, chunks);
        } else {
            let cand = if line_chunk.is_empty() {
                line.to_string()
            } else {
                format!("{line_chunk}\n{line}")
            };
            if estimate_token_count(&cand) > max_tokens && !line_chunk.is_empty() {
                chunks.push(line_chunk);
                line_chunk = line.to_string();
            } else {
                line_chunk = cand;
            }
        }
    }
    if !line_chunk.is_empty() {
        chunks.push(line_chunk);
    }
}

/// Splits a markdown note into semantic chunks each fitting within `max_tokens_per_chunk`.
pub fn chunk_markdown_note(note_text: &str, max_tokens_per_chunk: usize) -> Vec<String> {
    let trimmed = note_text.trim();
    if trimmed.is_empty() || max_tokens_per_chunk == 0 {
        return Vec::new();
    }
    if estimate_token_count(trimmed) <= max_tokens_per_chunk {
        return vec![trimmed.to_string()];
    }

    let mut chunks = Vec::new();
    let paragraphs: Vec<&str> = trimmed
        .split("\n\n")
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect();

    let mut current_chunk = String::new();

    for p in paragraphs {
        let p_tokens = estimate_token_count(p);
        if p_tokens > max_tokens_per_chunk {
            if !current_chunk.is_empty() {
                chunks.push(current_chunk.clone());
                current_chunk.clear();
            }
            split_large_paragraph(p, max_tokens_per_chunk, &mut chunks);
        } else {
            let cand = if current_chunk.is_empty() {
                p.to_string()
            } else {
                format!("{current_chunk}\n\n{p}")
            };
            if estimate_token_count(&cand) > max_tokens_per_chunk && !current_chunk.is_empty() {
                chunks.push(current_chunk);
                current_chunk = p.to_string();
            } else {
                current_chunk = cand;
            }
        }
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    chunks
}

/// Assembles system prompt, conversation history, and RAG context under budget limits.
pub fn assemble_rag_prompt(
    budget: &PromptBudget,
    system_prompt: &str,
    history: &[(String, String)],
    ranked_context: &[String],
    user_query: &str,
) -> AssembledPrompt {
    let system_str = truncate_to_tokens(system_prompt.trim(), budget.max_system_tokens);
    let system_tokens = estimate_token_count(&system_str);
    let query_tokens = estimate_token_count(user_query.trim());

    let remaining_for_rest = budget
        .max_total_tokens
        .saturating_sub(system_tokens + query_tokens);

    let history_budget = budget.max_history_tokens.min(remaining_for_rest);
    let mut conversation_history = Vec::new();
    let mut history_tokens = 0;
    for (speaker, text) in history.iter().rev() {
        let turn_tokens = estimate_token_count(speaker) + estimate_token_count(text);
        if history_tokens + turn_tokens <= history_budget {
            history_tokens += turn_tokens;
            conversation_history.push((speaker.clone(), text.clone()));
        } else {
            break;
        }
    }
    conversation_history.reverse();

    let remaining_for_context = remaining_for_rest.saturating_sub(history_tokens);
    let context_budget = budget.max_context_tokens.min(remaining_for_context);
    let mut context_snippets = Vec::new();
    let mut context_tokens = 0;

    for snippet in ranked_context {
        let s_trimmed = snippet.trim();
        if s_trimmed.is_empty() {
            continue;
        }
        let s_tokens = estimate_token_count(s_trimmed);
        if context_tokens + s_tokens <= context_budget {
            context_tokens += s_tokens;
            context_snippets.push(s_trimmed.to_string());
        } else if context_snippets.is_empty() && context_budget > 0 {
            let truncated = truncate_to_tokens(s_trimmed, context_budget);
            let t_tokens = estimate_token_count(&truncated);
            if t_tokens > 0 {
                context_tokens += t_tokens;
                context_snippets.push(truncated);
            }
            break;
        }
    }

    let total_estimated_tokens = (system_tokens + history_tokens + context_tokens + query_tokens)
        .min(budget.max_total_tokens);

    AssembledPrompt {
        system_prompt: system_str,
        conversation_history,
        context_snippets,
        total_estimated_tokens,
    }
}

/// Initializes a new `ThinkingState` with the specified phase message.
pub fn new_thinking_state(phase: &str) -> ThinkingState {
    ThinkingState {
        is_analyzing: true,
        phase_message: phase.to_string(),
    }
}

/// Marks the thinking state as finished and clears the phase message.
pub fn complete_thinking_state(state: &mut ThinkingState) {
    state.is_analyzing = false;
    state.phase_message.clear();
}
