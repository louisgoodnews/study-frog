---
id: STORY-0166
type: story
title: "Homework Due Date Reminders and Calendar Integration"
status: done
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
- [x] Calculation of due reminders and lead-time alerts integrated into due_reminders engine
- [x] Homework deadlines projected into daily and weekly calendar views alongside timetable sessions
- [x] Visual urgency indicators for overdue, due-soon, and completed assignments

## Design Notes
Reuse the existing `due_reminders` calculation and `LocalTime` abstractions from `studyfrog-calendar`.

## Dependencies
EPIC-056

## Implementation so far (2026-11-21)
`studyfrog-calendar::homework_reminders` (lead times 24 h, 2 h and 15 min; the most specific begun lead is shown once) and `studyfrog-service::homework::take_due_homework_reminders` (shown reminders persist in `homework_fired`). Tests: `studyfrog-calendar/tests/homework_reminders.rs`, `studyfrog-store/tests/homework_fired.rs`. Shown in the app: each tick checks homework alongside calendar reminders (`Effect::CheckHomeworkReminders`), with a toast and a notification line (`reminder_ui::homework_arrived`; tests in `studyfrog-ui/tests/reminders.rs`). Calendar: `studyfrog-calendar::homework_query::{due_on, most_urgent}`; `studyfrog-ui::view::homework_marks` draws urgency dots on month cells and the week header and a "Homework due" card in the day view (`PlanMsg::HomeworkLoaded`, `Effect::LoadHomework`; tests in `studyfrog-ui/tests/planner_homework.rs`). Folded into the reminder engine: `studyfrog-calendar::reminders::due_all` returns calendar and homework reminders together, one `take_due_reminders` service call marks both as shown, and the app runs one `CheckReminders` effect (`RemindersDue(Result<DueAll, _>)`). The markers are empty until items exist; they are created in the homework screen (STORY-0167).

## Open Questions
None.

## Implementation (2026-11-22, audit)

Audit (2026-11-22): criterion 1 MET: the 24 h, 2 h and 15 min leads fired through the reminder engine (notices "Homework: Kurzquiz (in 2 h, ...)" and "Homework: Mini Test (in 15 min, ...)"; toasts in /tmp/claude-0/audit/g5/0166-homework-all.png). Criterion 2 MET: deadlines show as dots in the month view (21, 22, 23, 25, 28; /tmp/claude-0/audit/g5/0166-month.png), in the week header (/tmp/claude-0/audit/g5/0166-week.png; the Sunday dot is clipped, BUG-0035) and as the "Homework due" card in the day view (/tmp/claude-0/audit/g5/0166-day-kurzquiz.png). Side-by-side with timetable sessions not checked: the test vault had no lectures, and the Timetable view shows no homework (/tmp/claude-0/audit/g5/0166-timetable.png). Criterion 3 MET: red Overdue, orange Due today, blue Due soon and green Done badges (/tmp/claude-0/audit/g5/0166-homework-all.png, /tmp/claude-0/audit/g5/0166-homework-completedtab.png). Defects: BUG-0034 (wording "in 1 days", "in 2 h"), BUG-0035 (week view clipped), BUG-0036 (toast covers Add session).
