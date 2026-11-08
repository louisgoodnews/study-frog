---
id: TASK-0237
type: task
title: Grade, store and list exam results
status: done
epic: EPIC-033
parents: [STORY-0127]
estimate: 3
labels: [m12, service]
acceptance_criteria:
  - "grade_exam reads the run's queue, answers and latest scores, computes and stores the result; the run history shows it; tested end to end on a fixture vault"
---
## Context
Part of STORY-0127.

## Description
`studyfrog-service/src/exam.rs`, `run_history.rs`.

## Acceptance Criteria
- [x] grade_exam reads the run's queue, answers and latest scores, computes and stores the result; the run history shows it; tested end to end on a fixture vault

## Design Notes
The latest score of an item counts; items are counted once even if the queue repeats them.

## Dependencies
TASK-0236.

## Open Questions
None.
