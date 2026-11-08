---
id: STORY-0126
type: story
title: Exam run data model and strict engine rules
status: done
epic: EPIC-033
parents: [EPIC-033]
estimate: 5
labels: [m12, core, rehearsal, store]
acceptance_criteria:
  - "RunMode::Exam exists with the persisted name exam, the run table accepts it (migration) and exam result records are stored and read back"
  - "The pure grade scale maps a percentage to the nine grades of the spec at the exact band boundaries, and the plan of an exam keeps only cards and questions, is practice-only and never requeues"
  - "The engine refuses flip, skip, rating and marks in an exam, refuses an empty or blank answer, locks an answer on submission and moves to the next item at once; property tests show no item is shown twice, skipped, rated or answered twice over arbitrary input sequences"
---
## Context
The exam rules must hold in the pure engine so every shell (window, control protocol) gets them for free.

## Description
Core types and the grade scale (TASK-0234), the store migration and records (TASK-0234), the engine rules with property tests (TASK-0235).

## Acceptance Criteria
- [x] RunMode::Exam exists with the persisted name exam, the run table accepts it (migration) and exam result records are stored and read back
- [x] The pure grade scale maps a percentage to the nine grades of the spec at the exact band boundaries, and the plan of an exam keeps only cards and questions, is practice-only and never requeues
- [x] The engine refuses flip, skip, rating and marks in an exam, refuses an empty or blank answer, locks an answer on submission and moves to the next item at once; property tests show no item is shown twice, skipped, rated or answered twice over arbitrary input sequences

## Design Notes
Rules live in `studyfrog-rehearsal/src/exam.rs`; the step function calls them during validation.

## Dependencies
STORY-0125.

## Open Questions
None.
