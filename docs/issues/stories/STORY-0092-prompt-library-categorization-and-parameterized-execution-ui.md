---
id: STORY-0092
type: story
title: Prompt library categorization and parameterized execution UI
status: done
epic: EPIC-030
parents: [EPIC-030]
estimate: 3
labels: [m12, agent, ui]
acceptance_criteria:
  - "The Assistant Prompts tab categorizes prompts into Content Generation, Interactive Tutoring & Exam, and Analysis"
  - "Parameterized prompts allow users to select configuration options (such as explanation depth or exam question count) prior to copying or running"
  - "Users can filter and search prompts by title, category, or intended workflow"
---

## Context
With the expansion of the prompt library from 6 static generation prompts to over 10 diverse prompts including interactive tutors, multi-level explainers, and diagnostic examiners, displaying all prompts in an unorganized flat list creates cognitive overload. Users need intuitive categories, search filtering, and the ability to customize prompt parameters before execution.

## Description
Upgrade the Prompts tab in `studyfrog-ui` and prompt representation in `studyfrog-service`:
1. Prompt Categories:
   - Introduce `PromptCategory` enum in `studyfrog-service::prompts`:
     - `ContentGeneration`: Flashcards, Questions, Whole Stack, Notes.
     - `InteractiveTutoring`: Oral Exam & Feedback, Socratic Dialogue.
     - `AnalysisAndInsights`: Multi-level Explanation, Concept Mapping, Summary, Critique.
2. Parameterized Prompt Customization:
   - Allow specific prompts to define optional parameters (e.g. explanation depth: `ELI5` / `Core` / `Technical`; exam intensity: `3 questions` / `5 questions`).
   - The UI displays clean selector controls (chips or combo boxes) that dynamically interpolate values into the generated prompt text.
3. UI Enhancements:
   - Update `crates/studyfrog-ui/src/view/agent_prompts.rs` with category tab pills, search input, and responsive prompt cards with copy, save, and direct run actions.

## Acceptance Criteria
- [x] The Assistant Prompts tab categorizes prompts into Content Generation, Interactive Tutoring & Exam, and Analysis
- [x] Parameterized prompts allow users to select configuration options (such as explanation depth or exam question count) prior to copying or running
- [x] Users can filter and search prompts by title, category, or intended workflow

## Design Notes
- Child task: TASK-0233: Move prompt library to standalone left sidebar navigation.
- Files: `crates/studyfrog-service/src/prompts.rs`, `crates/studyfrog-ui/src/view/agent_prompts.rs`, `crates/studyfrog-ui/src/prompts_ui.rs`.
- Adhere to UI guidelines and line count limits (keep all modified `.rs` files ≤ 250 lines).
- Release target: version 0.5.0 (Phase M12).

## Progress (2026-11-08)
Seen in the real binary (`--screen prompts`, and the new `--screen prompts:tutoring|analysis`): the category pills All, Create content, Tutoring and Analysis; the search box; "Be examined on a file" with the Questions choice (3 or 5 questions) and "Explain a file at three levels" with Depth (All three levels, Intuitive (ELI5), Core concepts, Technical), each with Copy to clipboard, Save as .md and Run with the assistant. The search now also matches the category name (it matched title and summary, the summary saying what the prompt is for); test `the_search_matches_the_category_name_too` in `tests/prompts_tab.rs`, placeholder "Search by title, category or what it is for".

## Dependencies
EPIC-030, STORY-0088, STORY-0089.

## Open Questions
None.
