---
id: TASK-0283
type: task
title: "Opt-in frame-time capture in the app window"
status: review
epic: EPIC-067
parents: ['STORY-0204']
estimate: 3
labels: [m14, perf, ui]
acceptance_criteria:
  - 'With an environment variable set, the app records the duration of each frame of the open screen and writes median, p95, worst and frame count on exit'
  - 'The summary is a pure function over the durations; the clock is passed in, not read inside it'
---

## Context
Part of STORY-0204. The egui test harness does not show what a window shows.

## Description
An opt-in recorder in the app shell, off by default, so normal runs are unaffected. The summary function is
tested on synthetic durations.

## Acceptance Criteria
- [ ] Opt-in capture writes the summary on exit
- [ ] The summary is pure and tested

## Design Notes
- Covers the UI-thread frame time as the app sees it. Does not cover GPU, compositor, or input latency.
- Needs a display to run; not verifiable in a headless container.

## Dependencies
STORY-0204. A display (the owner's machine).

## Open Questions
- Where in the app shell the timer starts.
