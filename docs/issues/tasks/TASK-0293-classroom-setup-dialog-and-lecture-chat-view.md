---
id: TASK-0293
type: task
title: "Classroom setup dialog and multi-participant lecture chat view in studyfrog-ui"
status: backlog
epic: EPIC-068
parents: ['STORY-0211']
estimate: 3
labels: [m14, ui]
acceptance_criteria:
  - 'A Classroom screen lets the user start a session by choosing a vault stack/notes/PDF, a local or remote model provider, and simulated student personas'
  - 'The lecture chat stream renders distinct role badges and styling for teacher, simulated students, and the user using palette_of(ui) tokens'
  - 'Controls allow advancing the lecture, prompting a simulated student question, requesting a task assignment, and submitting user questions asynchronously via the worker'
---

## Context
Part of `STORY-0211`. Provides the core interactive classroom-chat experience in StudyFrog's Elm-style UI architecture.

## Description
Implement the Classroom screen state, messages, worker effects, and view modules in `studyfrog-ui`:
- Session creation form (source picker, provider selector, student persona count/presets).
- Scrollable multi-participant chat transcript with role badges, speaker labels, and pending-turn indicator.
- Bottom composer bar for user questions/comments alongside quick lecture-flow buttons.

## Acceptance Criteria
- [ ] A Classroom screen lets the user start a session by choosing a vault stack/notes/PDF, a local or remote model provider, and simulated student personas
- [ ] The lecture chat stream renders distinct role badges and styling for teacher, simulated students, and the user using `palette_of(ui)` tokens
- [ ] Controls allow advancing the lecture, prompting a simulated student question, requesting a task assignment, and submitting user questions asynchronously via the worker

## Design Notes
- Keep every view/state module $\le 250$ LOC (Invariant P6) and use only `palette_of(ui)` tokens.

## Dependencies
- `STORY-0211`, `TASK-0290`

## Open Questions
None.
