---
id: EPIC-042
type: epic
title: Interactive AI content generation and parameter-guided prompt workflows
status: backlog
parents: []
labels: [m12, agent, docs, ui]
acceptance_criteria:
  - "The prompt library provides interactive prompts that guide AI models to consult the user for preferences before generating flashcards, questions, or notes from a source file"
  - "Interactive prompts guide the user through choices including relevant content scope, target language, difficulty level, and presentation style"
  - "The Assistant and Prompt Library UI support multi-turn interactive prompt sessions with option selection chips and staged confirmation before vault insertion"
---

## Context
When learners instruct an AI model to create study materials from a document or lecture slides, static one-shot generation often produces items that are misaligned with the learner's actual study goals: flashcards may cover peripheral trivia rather than exam-relevant topics, use the wrong language (e.g. English vs German), default to superficial difficulty, or adopt an undesirable format or style.

Instead of generating content blindly in one shot, prompts that instruct an AI to create study materials from a file should guide the AI through an interactive elicitation phase. The AI first asks the learner key clarifying questions:
1. *Scope & Relevance:* Which topics, chapters, or concepts in the document are actually relevant for the upcoming exam or study objective?
2. *Language:* Which language should the prompts and explanations use (or should technical terms remain bilingual)?
3. *Difficulty:* Foundational conceptual recall, intermediate application, or rigorous technical deep-dive?
4. *Style & Format:* Atomic flashcards (`???`), open questions with scoring rubrics, cloze deletions, or structured synthesis notes?

## Description
Deliver interactive, parameter-guided content generation workflows in StudyFrog's prompt library and assistant:

1. **Interactive Prompt Templates (`docs/prompts/`, `crates/studyfrog-service/src/prompts.rs`):**
   - Author interactive prompt templates that instruct the model to pause and engage in a guided intake interview before generating study content.
   - Specify interactive option schemas covering relevance filters, language selection, target cognitive depth (Bloom's taxonomy tiers), and card styling.
2. **Interactive Clarification Protocol (`crates/studyfrog-service`, `crates/studyfrog-ui`):**
   - Provide structured interactive prompt dialogs in the Assistant UI where the AI presents option choices as clickable pills or form selections.
   - Support iterative refinement of generated cards before committing them to the vault.
3. **Prompt Library Integration (`crates/studyfrog-ui/src/view/agent_prompts.rs`):**
   - Highlight interactive generation prompts in the Prompt Library with dedicated badges and configuration drawers.

## Acceptance Criteria
- [ ] The prompt library provides interactive prompts that guide AI models to consult the user for preferences before generating flashcards, questions, or notes from a source file
- [ ] Interactive prompts guide the user through choices including relevant content scope, target language, difficulty level, and presentation style
- [ ] The Assistant and Prompt Library UI support multi-turn interactive prompt sessions with option selection chips and staged confirmation before vault insertion

## Design Notes
Child stories:
- STORY-0137: Interactive parameter-guided content generation prompts in prompt library (estimate: 5)

Touchpoints across crates:
- `docs/prompts/`: interactive prompt markdown templates.
- `crates/studyfrog-service`: `prompts.rs` and assistant message execution.
- `crates/studyfrog-ui`: `view/agent_prompts.rs`, `view/agent.rs`, `view/agent_staged.rs`.
- Guardrails: P1 (no inherent `impl` blocks), P2 (≤ 250 lines per `.rs` file).

## Dependencies
- EPIC-028 (Extensibility and model integration).
- EPIC-030 (Prompt expansion).
- EPIC-037 (MCP server, client, and staged authoring tools).

## Open Questions
- None.
