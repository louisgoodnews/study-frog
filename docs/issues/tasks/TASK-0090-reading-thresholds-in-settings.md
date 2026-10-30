---
id: TASK-0090
type: task
title: Reading thresholds in app.toml
status: done
epic: EPIC-019
parents: [STORY-0035]
estimate: 3
labels: [m6, followup]
acceptance_criteria:
  - "reading_fast_percent, reading_slow_percent and reading_min_words with ranges and line-numbered errors"
  - "The Settings screen shows the values in force"
---

## Context
An open flag from STATUS.md.

## Description
Reading thresholds in app.toml.

## Acceptance Criteria
- [x] reading_fast_percent, reading_slow_percent and reading_min_words with ranges and line-numbered errors
- [x] The Settings screen shows the values in force

## Design Notes
`ReadingTuning` in core; `reading_params(wpm, tuning)`.

## Dependencies
None.

## Open Questions
None.
