---
id: TASK-0069
type: task
title: Practice orders
status: done
epic: EPIC-007
parents: [STORY-0033]
estimate: 5
labels: [m3, rehearsal]
acceptance_criteria:
  - "Sequential (library order), shuffled (seeded, reproducible), hardest/easiest first, stalest first, lowest predicted recall"
  - "Deterministic: same input and seed give the same queue; every candidate appears once"
  - "proptest: result is a permutation of the filtered input"
---

## Context
Orders other than due-first never hide items: a run goes through the whole filtered list until it is exhausted.

## Description
Practice orders.

## Acceptance Criteria
- [x] Sequential (library order), shuffled (seeded, reproducible), hardest/easiest first, stalest first, lowest predicted recall
- [x] Deterministic: same input and seed give the same queue; every candidate appears once
- [x] proptest: result is a permutation of the filtered input

## Design Notes
In-house SplitMix64 + Fisher-Yates (no rand crate, P10). Lowest-recall order uses srs::retrievability.

## Dependencies
TASK-0068.

## Open Questions
None.
