---
id: TASK-0308
type: task
title: "Windowed item distribution and re-balancing scheduler in studyfrog-calendar"
status: done
epic: EPIC-076
parents: ['STORY-0247']
estimate: 3
labels: [m14, calendar, srs]
acceptance_criteria:
  - 'A pure scheduler distributes all linked items across [max(today, exam_date - N weeks), exam_date - 1 day], prioritizing new and lowest-familiarity items in earlier days and spacing follow-up passes before exam day'
  - 'Respects daily review budgets and warns cleanly if the requested window cannot fit all items within the budget'
  - 'A re-balance function recomputes remaining days when items are added/removed, exam_date changes, or planned days are missed, verified by unit and property tests'
---

## Context
Part of `STORY-0247`. Converts an exam date, linked items, and a chosen preparation window into concrete daily study blocks.

## Description
Add a pure `exam_schedule` module in `studyfrog-calendar` implementing windowed item distribution, priority ordering (unstudied/weak first), daily cap handling, and re-balancing.

## Acceptance Criteria
- [x] A pure scheduler distributes all linked items across `[max(today, exam_date - N weeks), exam_date - 1 day]`, prioritizing new and lowest-familiarity items in earlier days and spacing follow-up passes before exam day
- [x] Respects daily review budgets and warns cleanly if the requested window cannot fit all items within the budget
- [x] A re-balance function recomputes remaining days when items are added/removed, `exam_date` changes, or planned days are missed, verified by unit and property tests

## Design Notes
- Keep the scheduler pure with injected `today` date (P2) and $\le 250$ LOC per file (P6).

## Dependencies
- `STORY-0247`, `TASK-0305`, `TASK-0307`

## Open Questions
None.
