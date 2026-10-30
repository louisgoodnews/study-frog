---
id: TASK-0039
type: task
title: Rehearsal screen
status: backlog
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
- [ ] Keyboard-first: space flips, 1-4 rate, s skips, p pauses
- [ ] Shows prompt and timer only (distraction-free)
- [ ] Every step goes through studyfrog-service::record_step

## Design Notes
Latency measured from engine timestamps, not from UI frames.

## Dependencies
TASK-0037, TASK-0073.

## Open Questions
None.
