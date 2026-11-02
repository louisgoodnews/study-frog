---
id: TASK-0180
type: task
title: Countdown and count up in the run state machine
status: done
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
- [x] The run plan has a mode (free, countdown with seconds, count up) that is stored with the run and read as free from older runs
- [x] A countdown ends the run at zero by itself, marks the open item as not answered and does not change the schedule from it
- [x] A count up ends when the last item is completed or the user ends it, and time paused is never counted
- [x] Property tests with the injected clock cover both modes and crash recovery restores a run in either mode

## Design Notes
Done: `studyfrog_core::pacing` (the type, the stored words, the limits 60 to 10 800 s), `RunPlan.pacing`, the event `TimeUp`, `Action::Tick` and the time-up rule in `studyfrog-rehearsal::step`, the clock helpers in `studyfrog-rehearsal::pacing` (`active_ms`, `remaining_ms`, `run_active_ms`), migration 16 and the store. The setup screen still always makes free runs and nothing sends ticks yet: that is TASK-0181. Crash recovery needed no change: a recovered run is `Aborted` and keeps its pacing.

## Dependencies
The specification task of this story.

## Open Questions
None.
