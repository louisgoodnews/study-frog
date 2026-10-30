---
id: TASK-0091
type: task
title: Shuffling is a switch of its own
status: done
epic: EPIC-019
parents: [STORY-0035]
estimate: 3
labels: [m6, followup]
acceptance_criteria:
  - "Any order can be shuffled; the effect on the schedule does not change"
  - "A limit cuts by priority and the cut items are shuffled; a sequential run samples at random"
  - "Stored shuffled runs are converted (schema v7)"
---

## Context
A decision from the owner review.

## Description
Shuffling is a switch of its own.

## Acceptance Criteria
- [x] Any order can be shuffled; the effect on the schedule does not change
- [x] A limit cuts by priority and the cut items are shuffled; a sequential run samples at random
- [x] Stored shuffled runs are converted (schema v7)

## Design Notes
`RunPlan.shuffle`; `Order::Shuffled` is gone. The Rehearse screen has a "Shuffle the order of the chosen items" checkbox.

## Dependencies
None.

## Open Questions
None.
