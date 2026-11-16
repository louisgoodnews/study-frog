//! Integration tests for RAG prompt budgeting and context assembly (TASK-0276).

use studyfrog_service::rag_pipeline::{
    AssembledPrompt, PromptBudget, ThinkingState, assemble_rag_prompt, chunk_markdown_note,
    complete_thinking_state, default_prompt_budget, estimate_token_count, new_thinking_state,
    truncate_to_tokens,
};

#[test]
fn default_budget_values() {
    let budget = default_prompt_budget();
    assert_eq!(budget.max_total_tokens, 1024);
    assert_eq!(budget.max_system_tokens, 150);
    assert_eq!(budget.max_history_tokens, 150);
    assert_eq!(budget.max_context_tokens, 768);

    let default_trait: PromptBudget = Default::default();
    assert_eq!(budget, default_trait);
}

#[test]
fn token_count_estimation() {
    assert_eq!(estimate_token_count(""), 0);
    assert_eq!(estimate_token_count("a"), 1);
    assert_eq!(estimate_token_count("abcd"), 1);
    assert_eq!(estimate_token_count("abcde"), 2);
    assert_eq!(estimate_token_count("hello world!"), 3);
}

#[test]
fn token_truncation() {
    let text = "This is a sentence designed to test truncation.";
    let truncated = truncate_to_tokens(text, 3);
    assert!(estimate_token_count(&truncated) <= 3);

    let exact = truncate_to_tokens("abcd", 1);
    assert_eq!(exact, "abcd");

    let zero = truncate_to_tokens("hello", 0);
    assert_eq!(zero, "");
}

#[test]
fn markdown_chunking_empty_and_small() {
    assert!(chunk_markdown_note("", 250).is_empty());
    assert!(chunk_markdown_note("   \n\n   ", 250).is_empty());

    let small = "# Header\n\nParagraph text here.";
    let chunks = chunk_markdown_note(small, 250);
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0], small);
}

#[test]
fn markdown_chunking_multiple_paragraphs() {
    let p1 = "First paragraph with some words for chunk testing.";
    let p2 = "Second paragraph with more words for chunk testing.";
    let p3 = "Third paragraph with extra details for chunk testing.";
    let note = format!("{p1}\n\n{p2}\n\n{p3}");

    let chunks = chunk_markdown_note(&note, 15);
    assert!(chunks.len() >= 2);
    for chunk in &chunks {
        assert!(estimate_token_count(chunk) <= 15);
    }
}

#[test]
fn markdown_chunking_oversized_paragraph() {
    let long_line = "word ".repeat(50);
    let chunks = chunk_markdown_note(&long_line, 10);
    assert!(chunks.len() > 1);
    for chunk in &chunks {
        assert!(estimate_token_count(chunk) <= 10);
    }
}

#[test]
fn assemble_prompt_within_budget() {
    let budget = default_prompt_budget();
    let sys = "You are a helpful study assistant.";
    let history = vec![
        ("user".to_string(), "What is ATP?".to_string()),
        (
            "assistant".to_string(),
            "ATP is adenosine triphosphate.".to_string(),
        ),
    ];
    let context = vec![
        "ATP is the energy currency of the cell.".to_string(),
        "Glycolysis produces 2 net ATP molecules.".to_string(),
    ];
    let query = "How is it produced?";

    let assembled: AssembledPrompt = assemble_rag_prompt(&budget, sys, &history, &context, query);
    assert_eq!(assembled.system_prompt, sys);
    assert_eq!(assembled.conversation_history.len(), 2);
    assert_eq!(assembled.context_snippets.len(), 2);
    assert!(assembled.total_estimated_tokens <= budget.max_total_tokens);
}

#[test]
fn assemble_prompt_truncates_oversized_system() {
    let mut budget = default_prompt_budget();
    budget.max_system_tokens = 5;
    let long_sys =
        "You are an AI assistant designed to help students learn effectively and quickly.";
    let assembled = assemble_rag_prompt(&budget, long_sys, &[], &[], "help");
    assert!(estimate_token_count(&assembled.system_prompt) <= 5);
}

#[test]
fn assemble_prompt_history_sliding_window() {
    let mut budget = default_prompt_budget();
    budget.max_history_tokens = 10;

    let history = vec![
        ("user".to_string(), "Old question 1".to_string()),
        ("assistant".to_string(), "Old answer 1".to_string()),
        ("user".to_string(), "Recent question".to_string()),
        ("assistant".to_string(), "Recent answer".to_string()),
    ];

    let assembled = assemble_rag_prompt(&budget, "Sys", &history, &[], "Now");
    assert!(assembled.conversation_history.len() < history.len());
    let last = assembled.conversation_history.last().unwrap();
    assert_eq!(last.1, "Recent answer");
}

#[test]
fn assemble_prompt_caps_context_snippets() {
    let mut budget = default_prompt_budget();
    budget.max_context_tokens = 20;

    let snippets = vec![
        "Short snippet one.".to_string(),
        "Short snippet two.".to_string(),
        "This third snippet is way too long and cannot fit into the remaining twenty token budget."
            .to_string(),
    ];

    let assembled = assemble_rag_prompt(&budget, "Sys", &[], &snippets, "Query");
    assert!(assembled.context_snippets.len() <= 2);
    let total_context_tokens: usize = assembled
        .context_snippets
        .iter()
        .map(|s| estimate_token_count(s))
        .sum();
    assert!(total_context_tokens <= 20);
}

#[test]
fn thinking_state_lifecycle() {
    let mut state: ThinkingState = new_thinking_state("Thinking / Analyzing vault...");
    assert!(state.is_analyzing);
    assert_eq!(state.phase_message, "Thinking / Analyzing vault...");

    complete_thinking_state(&mut state);
    assert!(!state.is_analyzing);
    assert!(state.phase_message.is_empty());
}
