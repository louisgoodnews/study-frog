---
id: TASK-0181
type: task
title: Mode and timer in run setup and on the rehearsal screen
status: done
epic: EPIC-023
parents: [STORY-0070]
estimate: 3
labels: [m10, ui]
acceptance_criteria:
  - "Run setup offers the mode and, for a countdown, presets and a custom time"
  - "The rehearsal screen shows the remaining or elapsed time, warns at one minute and ten seconds in text, and announces it to screen readers"
  - "At zero the review opens, and the accessibility audit passes"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Add the setup control and the timer to the screens.

## Acceptance Criteria
- [x] Run setup offers the mode and, for a countdown, presets and a custom time
- [x] The rehearsal screen shows the remaining or elapsed time, warns at one minute and ten seconds in text, and announces it to screen readers
- [x] At zero the review opens, and the accessibility audit passes

## Design Notes
None.

## Dependencies
The engine task of this story.

## Open Questions
None.
