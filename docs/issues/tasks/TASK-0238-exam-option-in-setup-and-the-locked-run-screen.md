---
id: TASK-0238
type: task
title: Exam option in the setup and the locked run screen
status: done
epic: EPIC-033
parents: [STORY-0128]
estimate: 3
labels: [m12, ui]
acceptance_criteria:
  - "Exam is selectable in the setup with its explanation, and the run screen offers only the answer field and Submit (disabled while blank) without skip, show-answer, marks or ratings; tested in the harness"
---
## Context
Part of STORY-0128.

## Description
`view/setup_how.rs`, `view/run.rs`, `view/run_card.rs`, `run_update.rs`.

## Acceptance Criteria
- [x] Exam is selectable in the setup with its explanation, and the run screen offers only the answer field and Submit (disabled while blank) without skip, show-answer, marks or ratings; tested in the harness

## Design Notes
Esc does not stop an exam; only the Stop button does, and a stopped exam is not graded.

## Dependencies
TASK-0237.

## Open Questions
None.
