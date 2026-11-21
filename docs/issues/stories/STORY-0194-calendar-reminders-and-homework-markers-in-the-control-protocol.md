---
id: STORY-0194
type: story
title: "Calendar, reminders and homework markers in the control protocol"
status: backlog
epic: EPIC-064
parents: ['EPIC-064']
estimate: 3
labels: [m14, tooling, ui]
acceptance_criteria:
  - 'calendar view with month, week, day or timetable changes the view, and calendar period with a date moves to it'
  - 'reminders due answers with the calendar and homework reminders due now, without marking them shown'
  - 'calendar homework with a date answers with the homework due that day and its urgency'
---

## Context
The calendar's view modes, its homework markers (STORY-0166) and the reminder engine that now covers homework (due_all) cannot be reached from the command line.

## Description
reminders due must not mark anything shown, but take_due_reminders does; the command should call due_all on a read of the sessions and items.

## Acceptance Criteria
- [ ] calendar view with month, week, day or timetable changes the view, and calendar period with a date moves to it
- [ ] reminders due answers with the calendar and homework reminders due now, without marking them shown
- [ ] calendar homework with a date answers with the homework due that day and its urgency

## Design Notes
None.

## Dependencies
EPIC-064, and the features named above.

## Open Questions
None.
