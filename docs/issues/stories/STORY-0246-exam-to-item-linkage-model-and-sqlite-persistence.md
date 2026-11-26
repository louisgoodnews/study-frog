---
id: STORY-0246
type: story
title: "Exam-to-item linkage model and SQLite persistence for the Exam Scheduler"
status: done
epic: EPIC-076
parents: ['EPIC-076']
estimate: 5
labels: [m14, calendar, store]
acceptance_criteria:
  - 'Pure domain types model an exam plan: target date, preparation window mode (UserWeeks or Proposed), linked targets (stacks, individual cards, questions, notes), and generated daily schedule blocks'
  - 'A SQLite migration and repository in studyfrog-store persist and reload exam plans, item links, and scheduled blocks atomically across restarts'
  - 'Unit and store round-trip tests verify creation, updates, item linking/unlinking, and cascading deletes'
---

## Context
To automatically schedule study material for an upcoming exam (`EPIC-076`), StudyFrog needs a clean domain model and local SQLite persistence linking an exam to its stacks, flashcards, questions, and notes alongside the chosen preparation window.

## Description
Add pure domain types in `studyfrog-core` / `studyfrog-calendar` and SQLite persistence in `studyfrog-store` for exam plans, target links (stacks and individual items), preparation-window configuration, and generated schedule blocks.

## Acceptance Criteria
- [x] Pure domain types model an exam plan: target date, preparation window mode (`UserWeeks` or `Proposed`), linked targets (stacks, individual cards, questions, notes), and generated daily schedule blocks
- [x] A SQLite migration and repository in `studyfrog-store` persist and reload exam plans, item links, and scheduled blocks atomically across restarts
- [x] Unit and store round-trip tests verify creation, updates, item linking/unlinking, and cascading deletes

## Design Notes
- Follow Invariant P1 (zero inherent `impl` blocks), P6 ($\le 250$ LOC), and P8 (local-first SQLite persistence).

## Dependencies
- `EPIC-076`
- Child tasks: `TASK-0305`, `TASK-0306`

## Open Questions
None.
