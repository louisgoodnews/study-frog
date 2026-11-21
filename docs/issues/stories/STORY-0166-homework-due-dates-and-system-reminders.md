---
id: STORY-0166
type: story
title: "Homework Due Date Reminders and Calendar Integration"
status: in_progress
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

## Implementation so far (2026-11-21)
`studyfrog-calendar::homework_reminders` (lead times 24 h, 2 h and 15 min; the most specific begun lead is shown once) and `studyfrog-service::homework::take_due_homework_reminders` (shown reminders persist in `homework_fired`). Tests: `studyfrog-calendar/tests/homework_reminders.rs`, `studyfrog-store/tests/homework_fired.rs`. Still to do: show these reminders in the app's notification flow.

## Open Questions
None.
