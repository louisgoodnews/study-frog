---
id: TASK-0039
type: task
title: Rehearsal screen
status: done
epic: EPIC-008
parents: [STORY-0017]
estimate: 5
labels: [m4, ui, rehearsal]
acceptance_criteria:
  - "Keyboard-first: space flips, 1-4 rate, s skips, p pauses"
  - "Shows prompt and timer only (distraction-free)"
  - "Every step goes through studyfrog-service::record_step"
---

## Context
docs/research/ui-guidelines.md.

## Description
Rehearsal screen.

## Acceptance Criteria
- [x] Keyboard-first: space flips, 1-4 rate, s skips, p pauses
- [x] Shows prompt and timer only (distraction-free)
- [x] Every step goes through studyfrog-service::record_step

## Design Notes
Latency is engine time: the shell writes the egui time into `State::now` each frame, `update` turns it into the engine's millisecond timestamps and calls the pure `step` itself, and each step becomes a `RecordStep` effect handled by `studyfrog-service`. Content is loaded lazily with a 3-item look-ahead. Key map and behaviour: docs/specs/rehearsal-run.md. 8 logic tests over a real vault (spaced run saved into the schedule, practice-only run leaves it alone, stray keys ignored, pause excluded from latency, abort, unreadable item), 7 headless widget tests including keyboard-only operation and an accessibility audit of every state.

## Dependencies
TASK-0037, TASK-0073.

## Open Questions
None.
