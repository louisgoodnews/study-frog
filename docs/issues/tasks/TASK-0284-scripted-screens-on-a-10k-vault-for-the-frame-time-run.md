---
id: TASK-0284
type: task
title: "Scripted visit of the rehearsal card, reports and console on a 10k-note vault"
status: review
epic: EPIC-067
parents: ['STORY-0204']
estimate: 2
labels: [m14, perf, ui]
acceptance_criteria:
  - 'A script generates a 10k-note vault, opens the rehearsal card, the reports screen and the console, and records each screen''s frame summary'
  - 'The script states which of the three could not be measured in the environment it ran in'
---

## Context
Part of STORY-0204. The 10k-note vault matches the store benchmark.

## Description
Uses the opt-in capture of TASK-0283 and a fixed interaction script. Needs either a person at a window or a
screenshot harness that can drive a timed loop.

## Acceptance Criteria
- [ ] The three screens are measured on a generated 10k-note vault
- [ ] Unmeasured screens are named in the output

## Design Notes
- Whether the `--screenshot` demos can drive a timed loop is open (STORY-0204).

## Dependencies
TASK-0283; a machine with a display.

## Open Questions
- Whether the console screen can be opened by the control protocol in the same run.
