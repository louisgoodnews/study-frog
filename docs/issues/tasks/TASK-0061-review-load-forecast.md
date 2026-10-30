---
id: TASK-0061
type: task
title: Review-load forecast and backtest
status: done
epic: EPIC-015
parents: [STORY-0029]
estimate: 5
labels: [m7, report]
acceptance_criteria:
  - "Expected reviews per day projected from schedule, growth, lapses and the new-item pace"
  - "The model is estimated from the history"
  - "Backtest compares a forecast with what happened (weekly error, bias); a seeded simulated learner supplies history when the real one is short"
---

## Context
The exit criterion asks for a measured forecast error.

## Description
Review-load forecast and backtest.

## Acceptance Criteria
- [x] Expected reviews per day projected from schedule, growth, lapses and the new-item pace
- [x] The model is estimated from the history
- [x] Backtest compares a forecast with what happened (weekly error, bias); a seeded simulated learner supplies history when the real one is short

## Design Notes
`report::forecast`, `backtest`, `simulate`. Measured on simulated histories: weekly totals off by 9 to 11 %, forecast 8 to 11 % low. Not yet measured on a real history, because none exists yet.

## Dependencies
TASK-0059.

## Open Questions
None.
