---
id: STORY-0137
type: story
title: "Interactive parameter-guided content generation prompts in prompt library"
status: done
epic: EPIC-042
parents: [EPIC-042]
estimate: 5
labels: [m12, agent, docs, ui]
acceptance_criteria:
  - "The prompt library includes interactive file-to-content prompts instructing the AI model to conduct an intake interview before generating study items"
  - "The interactive prompt guides the user through choices of relevant content scope, target language, difficulty tier, and card/note style"
  - "The prompt output produces clean, StudyFrog-compatible CSV/Markdown blocks matching the user's chosen parameters, ready for one-click staging or vault import"
---

## Context
When learners use StudyFrog's Prompt Library or AI Assistant to turn lecture notes, textbook chapters, or PDF documents into study items (flashcards, questions, and notes), current static generation prompts immediately output items in one shot.

This frequently leads to low-quality or irrelevant items: the AI might extract trivial definitions, choose the wrong language (e.g. English when the student studies in German), pick an inappropriate difficulty level (superficial trivia vs analytical synthesis), or adopt a style that doesn't fit the learner's study habits.

Learners need content generation prompts in the Prompt Library that are structured to interactively interview the user before generating items.

## Description
Deliver interactive content generation prompts in StudyFrog's Prompt Library:

1. **Interactive Prompt Template Structure (`docs/prompts/interactive_extract.md`):**
   - Provide a system-prompted workflow where the AI first reads the attached document and responds with an intake menu:
     - **Relevance & Focus:** Asks the user which sections, themes, or core concepts are relevant for their current study goals (or provides a bulleted outline of the document for the user to pick from).
     - **Language:** Offers language choices (e.g., German, English, bilingual definitions).
     - **Difficulty:** Foundational/introductory (definitions and key terms), intermediate (conceptual mechanisms and comparisons), or advanced (critical analysis, edge cases, formula derivations).
     - **Style & Output Kind:** Atomic two-sided flashcards (`???`), open questions with rubric scoring, cloze deletion sentences (`{{c1::...}}`), or synthesis concept notes.
2. **Interactive Multi-Turn Flow in Prompt Library & Assistant (`crates/studyfrog-ui`):**
   - In the Prompt Library (`view/agent_prompts.rs`), add the `Interactive Content Extraction` prompt under the Content Generation category with quick-launch buttons.
   - When run in the Assistant view (`view/agent.rs`), the initial message displays the structured intake questionnaire with clickable option suggestions.
3. **Structured StudyFrog Serialization:**
   - Once the user confirms their choices, the AI generates the study items strictly formatted according to StudyFrog's CSV or Markdown schema (`stack,kind,title,front,back,tags`) for instant, error-free staging and import.

## Acceptance Criteria
- [x] The prompt library includes interactive file-to-content prompts instructing the AI model to conduct an intake interview before generating study items
- [x] The interactive prompt guides the user through choices of relevant content scope, target language, difficulty tier, and card/note style
- [x] The prompt output produces clean, StudyFrog-compatible CSV/Markdown blocks matching the user's chosen parameters, ready for one-click staging or vault import

## Design Notes
- Store prompt templates as versioned Markdown files under `docs/prompts/` registered in `crates/studyfrog-service/src/prompts.rs`.
- Ensure prompts clearly instruct the LLM not to emit preamble or conversation chatter once generating the final CSV/Markdown block.
- Follow architectural rules: zero inherent `impl` blocks (P1) and ≤ 250 lines per file (P2).

## Dependencies
- EPIC-042 (Interactive AI content generation).
- STORY-0092 (Prompt library categorization and execution UI).

## Open Questions
- None.
