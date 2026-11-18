---
id: STORY-0166
type: story
title: "Homework Due Date Reminders and Calendar Integration"
status: backlog
epic: EPIC-056
parents: ['EPIC-056']
estimate: 5
labels: [m14, calendar, app]
acceptance_criteria:
  - 'Calculation of due reminders and lead-time alerts integrated into due_reminders engine'
  - 'Homework deadlines projected into daily and weekly calendar views alongside timetable sessions'
  - 'Visual urgency indicators for overdue, due-soon, and completed assignments'
---

## Context
Students need active awareness of upcoming deadlines through configurable reminders and visual alignment with their daily course schedules.

## Description
Integrate homework assignments into the reminder and calendar systems:
- Extend `due_reminders` in `studyfrog-calendar` to generate alerts based on homework due timestamps and student-configured lead times.
- Project homework deadlines into the calendar view as milestone markers on due dates.
- Compute dynamic urgency states (e.g. Overdue, Due Today, Due Tomorrow, Later).

## Acceptance Criteria
- [ ] Calculation of due reminders and lead-time alerts integrated into due_reminders engine
- [ ] Homework deadlines projected into daily and weekly calendar views alongside timetable sessions
- [ ] Visual urgency indicators for overdue, due-soon, and completed assignments

## Design Notes
Reuse the existing `due_reminders` calculation and `LocalTime` abstractions from `studyfrog-calendar`.

## Dependencies
EPIC-056

## Open Questions
None.
