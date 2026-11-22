---
id: STORY-0204
type: story
title: "Frame time of the rehearsal card, reports and console measured in the running window"
status: backlog
epic: EPIC-067
parents: ['EPIC-067']
estimate: 5
labels: [m14, perf, ui]
acceptance_criteria:
  - 'The frame time of the rehearsal card, the reports screen and the console on a large vault is recorded from the running window, not from the egui test harness'
  - 'Each screen reports the median, p95 and worst frame over a fixed script of interactions, with the machine identity'
  - 'The frame budget that applies is set from the owner''s baseline, not in this story'
---

## Context
The benchmarks cover the maths and the data paths, not what the user sees. EPIC-067 asks for frame time of the
interactive screens, measured in the window (EPIC-067 Decisions).

## Description
- An opt-in capture in the app records the time of each frame of the open screen and writes a summary on exit.
- A scripted visit opens the three screens on a generated 10k-note vault and records each summary.

## Acceptance Criteria
- [ ] Frame time of the rehearsal card, the reports screen and the console on a large vault is recorded from the running window
- [ ] Each screen reports median, p95 and worst frame over a fixed script of interactions, with the machine identity
- [ ] The frame budget is set from the owner's baseline, not here

## Design Notes
- What this covers: CPU time of the frame on the UI thread, as seen by the app.
- What this does not cover: GPU time, compositor and display latency, and input latency. Say so in the report.
- The measurement needs a person at a window, or a screenshot harness that drives a timed frame loop. The app
  already has `--screenshot` demos; whether one can drive a timed loop is an open question.
- The clock is passed into the summary function, not read inside it (P2).

## Dependencies
TASK-0283, TASK-0284. A machine with a display (the owner's machine, or a container with a virtual display).

## Open Questions
- Whether the `--screenshot` harness can drive a fixed interaction script with timing.
