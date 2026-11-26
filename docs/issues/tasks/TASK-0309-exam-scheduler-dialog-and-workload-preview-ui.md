---
id: TASK-0309
type: task
title: "Exam Scheduler dialog with stack/item picker, Custom vs Proposed weeks, and load preview"
status: backlog
epic: EPIC-076
parents: ['STORY-0248']
estimate: 3
labels: [m14, ui]
acceptance_criteria:
  - 'Users can create or edit an exam plan, set its target date, and attach whole stacks and/or individual cards, questions, and notes'
  - 'Users can toggle between Custom weeks (stepper/input) and StudyFrog proposed (N weeks) with the factor breakdown displayed'
  - 'A live workload preview shows daily/weekly item counts, capacity warnings if tight, and Apply / Re-balance actions'
---

## Context
Part of `STORY-0248`. Provides the interactive UI for configuring an exam, linking study material, and previewing the generated schedule.

## Description
Build the Exam Scheduler dialog/panel in `studyfrog-ui` (using redesign cards and `palette_of(ui)` tokens) wired to `studyfrog-service` for stats-based week estimation, preview generation, and persistence.

## Acceptance Criteria
- [ ] Users can create or edit an exam plan, set its target date, and attach whole stacks and/or individual cards, questions, and notes
- [ ] Users can toggle between Custom weeks (stepper/input) and StudyFrog proposed (N weeks) with the factor breakdown displayed
- [ ] A live workload preview shows daily/weekly item counts, capacity warnings if tight, and Apply / Re-balance actions

## Design Notes
- Split state, dialog view, and preview rendering into modular files $\le 250$ LOC (P6).

## Dependencies
- `STORY-0248`, `TASK-0306`, `TASK-0308`

## Open Questions
None.
