---
id: TASK-0058
type: task
title: Achievements screen and announcements
status: done
epic: EPIC-014
parents: [STORY-0027]
estimate: 3
labels: [m7, ui]
acceptance_criteria:
  - "Screen lists unlocked achievements with their date and the others with a progress bar and value / threshold"
  - "A new unlock shows in the status bar when a run ends or the screen opens"
  - "Accessibility audit passes"
---

## Context


## Description
Achievements screen and announcements.

## Acceptance Criteria
- [x] Screen lists unlocked achievements with their date and the others with a progress bar and value / threshold
- [x] A new unlock shows in the status bar when a run ends or the screen opens
- [x] Accessibility audit passes

## Design Notes
The announcement uses the status-bar notice (dismissable), not a separate toast layer. Checks run when the screen opens, when a run ends and after a suggested rating was applied.

## Dependencies
TASK-0057.

## Open Questions
None.
