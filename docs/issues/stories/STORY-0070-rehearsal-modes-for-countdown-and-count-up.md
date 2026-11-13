---
id: STORY-0070
type: story
title: "Rehearsal modes for countdown and count up"
status: done
epic: EPIC-023
parents: [EPIC-023]
estimate: 8
labels: [m10, rehearsal, ui]
acceptance_criteria:
  - "A run can be set up as a countdown with a time amount the user chooses, and it ends by itself when the time is up and opens the review of the performance"
  - "A run can be set up as a count up, which shows the elapsed time and goes on until the last item of the list is completed or the user ends it"
  - "The mode and the time used are stored with the run and shown in the review and the reports"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Give the user two more ways to rehearse besides the free run: against the clock, and open-ended with the time counted.

## Acceptance Criteria
- [x] A run can be set up as a countdown with a time amount the user chooses, and it ends by itself when the time is up and opens the review of the performance
- [x] A run can be set up as a count up, which shows the elapsed time and goes on until the last item of the list is completed or the user ends it
- [x] The mode and the time used are stored with the run and shown in the review and the reports

## Design Notes
The state machine (`studyfrog-rehearsal`) is pure and takes an injected clock, so the modes are added as a field of the run plan: Free (today's behaviour), Countdown with a number of seconds, and CountUp. In a countdown the user sets a fixed time (presets such as 5, 10, 15 and 25 minutes and a custom amount); the screen shows the remaining time, warns at one minute and ten seconds in text and sound-free cues (not by colour alone, and announced to screen readers) and, at zero, forces the end: the item on screen counts as not answered, the run ends and the review of the run's performance opens at once (the review after each run already exists). If the queue runs out before the time, the run ends early. Pausing stops the clock, as paused time is already left out of the latency. In a count up the screen shows the elapsed time, there is no limit and the run ends when the last item of the list has been completed; the user can still end it early, which opens the same review. The review and the reports show the mode, the time planned and the time used, and the answer rate per minute, so a run against the clock can be compared with another. The run plan is stored with the run, so a stored plan from an older version reads as Free.

Tasks: TASK-0179, TASK-0180, TASK-0181, TASK-0182.

## Dependencies
None. Crash recovery of runs (EPIC-007) must cover the new fields.

## Open Questions
Answered by the owner on 2026-11-02: at the end of a countdown the item on screen counts as not answered, without a rating, so it does not change the schedule. Still open: may a countdown be extended once it has started? Proposal: no.
