---
id: TASK-0087
type: task
title: Store answer scores with the run
status: done
epic: EPIC-019
parents: [STORY-0035]
estimate: 3
labels: [m6, followup]
acceptance_criteria:
  - "Schema v6 answer_score"
  - "The comparison is stored when it finishes; the review shows the match per item"
---

## Context
An open flag from STATUS.md.

## Description
Store answer scores with the run.

## Acceptance Criteria
- [x] Schema v6 answer_score
- [x] The comparison is stored when it finishes; the review shows the match per item

## Design Notes
`store::answer_scores`, `service::compare::record_answer_score`. A failed save is noted next to the engine name, not hidden.

## Dependencies
None.

## Open Questions
None.
