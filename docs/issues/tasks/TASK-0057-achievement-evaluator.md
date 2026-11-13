---
id: TASK-0057
type: task
title: "Evaluator and unlock memory"
status: done
epic: EPIC-014
parents: [STORY-0026]
estimate: 5
labels: [m7, achieve, store, service]
acceptance_criteria:
  - "Statistics per vault and per stack from the review history; the streak follows the local day"
  - "Each rule unlocks once and keeps its first time (schema v5 `achievement_unlock`)"
  - "Stacks with no history still show their goals"
---

## Context
Evaluation must be repeatable.

## Description
Evaluator and unlock memory.

## Acceptance Criteria
- [x] Statistics per vault and per stack from the review history; the streak follows the local day
- [x] Each rule unlocks once and keeps its first time (schema v5 `achievement_unlock`)
- [x] Stacks with no history still show their goals

## Design Notes
`achieve::stats` and `achieve::evaluate` are pure and property-tested; `service::achievements` reads the history and stores new unlocks; checking twice unlocks nothing new.

## Dependencies
TASK-0056.

## Open Questions
None.
