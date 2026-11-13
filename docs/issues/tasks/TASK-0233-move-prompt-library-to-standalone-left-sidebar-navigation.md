---
id: TASK-0233
type: task
title: "Move prompt library to standalone left sidebar navigation"
status: done
epic: EPIC-030
parents: [STORY-0092]
estimate: 2
labels: [m12, ui]
acceptance_criteria:
  - "The prompt library is accessible directly as a dedicated screen (Screen::Prompts) from the left sidebar navigation under an appropriate group"
  - "The sidebar navigation highlights the Prompts item when active and supports history back/forward navigation"
  - "The Assistant screen remains focused on AI connections, vault tasks, and exams, with prompts smoothly migrated or cross-linked"
---

## Context
Currently, the Prompt Library is embedded as a fourth tab inside the Assistant screen (`Screen::Agent`). Users who want to browse, copy, export, or run prompts must first open Assistant and click the Prompts tab. As the prompt library expands into a central hub for study prompts, multi-level explanations, exams, and Socratic dialogues, it should have its own dedicated first-class navigation item in the left sidebar.

## Description
Promote the prompt library to a first-class screen in StudyFrog:
1. Add `Screen::Prompts` to `crates/studyfrog-ui/src/state.rs`.
2. Add `(Screen::Prompts, "Prompts")` to the left sidebar navigation in `crates/studyfrog-ui/src/view/sidebar.rs` (placed under the Study or Insight group, or alongside Assistant).
3. Connect screen routing in `crates/studyfrog-ui/src/view/mod.rs` to render `prompts::draw` when `Screen::Prompts` is active.
4. Update `crates/studyfrog-ui/src/appearance_ui.rs`, `crates/studyfrog-ui/src/help_ui.rs`, and `crates/studyfrog-ui/src/app_shot.rs` to support `Screen::Prompts`.
5. Keep a lightweight link or redirect in the Assistant screen so existing habits are preserved without duplication.

## Acceptance Criteria
- [x] The prompt library is accessible directly as a dedicated screen (Screen::Prompts) from the left sidebar navigation under an appropriate group
- [x] The sidebar navigation highlights the Prompts item when active and supports history back/forward navigation
- [x] The Assistant screen remains focused on AI connections, vault tasks, and exams, with prompts smoothly migrated or cross-linked

## Design Notes
- Files to touch:
  - `crates/studyfrog-ui/src/state.rs`: add `Screen::Prompts`.
  - `crates/studyfrog-ui/src/view/sidebar.rs`: add `(Screen::Prompts, "Prompts")` to `GROUPS`.
  - `crates/studyfrog-ui/src/view/mod.rs`: dispatch `Screen::Prompts`.
  - `crates/studyfrog-ui/src/view/agent_prompts.rs`: extract or adjust view rendering for standalone screen presentation.
- Ensure all modified files remain under the 250-line limit (rule P2).
- Release target: version 0.5.0 (Phase M12).

## Progress (2026-11-08)
Seen in the real binary: **Prompts** is its own entry in the sidebar (after Assistant) and is highlighted on the Prompts screen (`--screen prompts`); the back arrow is active after navigating there. The Assistant screen (`--screen agent`) has Connections, Work the vault, Exam me and a Prompts tab that only points to the screen ("Open Prompts", test `the_assistant_tab_points_to_the_screen_and_the_screen_is_in_the_sidebar`). Back and forward over the history are covered by the history tests.

## Dependencies
STORY-0092.

## Open Questions
None.
