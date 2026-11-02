---
id: TASK-0180
type: task
title: Countdown and count up in the run state machine
status: backlog
epic: EPIC-023
parents: [STORY-0070]
estimate: 5
labels: [m10, rehearsal, core, store]
acceptance_criteria:
  - "The run plan has a mode (free, countdown with seconds, count up) that is stored with the run and read as free from older runs"
  - "A countdown ends the run at zero by itself, marks the open item as not answered and does not change the schedule from it"
  - "A count up ends when the last item is completed or the user ends it, and time paused is never counted"
  - "Property tests with the injected clock cover both modes and crash recovery restores a run in either mode"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Add the mode to the pure engine, the persistence and the recovery.

## Acceptance Criteria
- [ ] The run plan has a mode (free, countdown with seconds, count up) that is stored with the run and read as free from older runs
- [ ] A countdown ends the run at zero by itself, marks the open item as not answered and does not change the schedule from it
- [ ] A count up ends when the last item is completed or the user ends it, and time paused is never counted
- [ ] Property tests with the injected clock cover both modes and crash recovery restores a run in either mode

## Design Notes
None.

## Dependencies
The specification task of this story.

## Open Questions
None.
