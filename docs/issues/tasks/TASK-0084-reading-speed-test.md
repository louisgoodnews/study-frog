---
id: TASK-0084
type: task
title: Reading-speed test
status: done
epic: EPIC-019
parents: [STORY-0035]
estimate: 3
labels: [m6, rehearsal, ui]
acceptance_criteria:
  - "Settings has a test: read lorem ipsum text, press done, get words per minute"
  - "The result is kept in app.toml as `reading_wpm` (50 to 1000) and can be reset to the default"
  - "Runs and the review use the measured speed as the baseline"
---

## Context
The 238 wpm average is a guess about the user; measuring it makes fast and slow reads meaningful.

## Description
Reading-speed test.

## Acceptance Criteria
- [x] Settings has a test: read lorem ipsum text, press done, get words per minute
- [x] The result is kept in app.toml as `reading_wpm` (50 to 1000) and can be reset to the default
- [x] Runs and the review use the measured speed as the baseline

## Design Notes
`ui::reading_test` (pure; time from `State::now`), `rehearsal::reading::measured_wpm` and `reading_params`, `config::app`. Lorem ipsum is Latin-looking, so it reads a little slower than real prose; the speed is a baseline for comparison, not a benchmark. A run keeps the speed it started with.

## Dependencies
TASK-0082.

## Open Questions
None.
