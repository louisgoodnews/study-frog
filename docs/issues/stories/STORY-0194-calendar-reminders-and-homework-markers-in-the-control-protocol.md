---
id: STORY-0194
type: story
title: "Calendar, reminders and homework markers in the control protocol"
status: review
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
- [x] calendar view with month, week, day or timetable changes the view, and calendar period with a date moves to it
- [x] reminders due answers with the calendar and homework reminders due now, without marking them shown
- [x] calendar homework with a date answers with the homework due that day and its urgency

## Design Notes
None.

## Implementation
- `calendar view MODE` sets the view (Planner SetViewMode) after the screen is opened; `calendar period DATE` moves the visible month by the months between and selects the day; `calendar homework DATE` selects the day and answers its homework with the urgency (`homework_query::urgency`).
- `reminders due` runs `Effect::PeekReminders` -> `studyfrog_service::reminders::due_now` (the same `due_all` as the take, without the marking) and answers `Planner.due_peek`.
- Opening the calendar now also loads the homework (for the markers and the answer).
- Limit: the reminder test uses a sample vault that may have no due reminders, so the no-marking rule is only partly exercised; the homework urgency and the view changes are not yet checked in the running app.

## Dependencies
EPIC-064, and the features named above.

## Open Questions
None.
