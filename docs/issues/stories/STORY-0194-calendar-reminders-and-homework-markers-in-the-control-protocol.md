---
id: STORY-0194
type: story
title: "Calendar, reminders and homework markers in the control protocol"
status: done
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
- `calendar session add TITLE --start "YYYY-MM-DD HH:MM" --minutes N` (write) creates a session through the planner's own path: the day and month shown, `PlanMsg::New`, the form's Title, Date, Time and Duration, then `PlanMsg::Save`; the answer is the created session. Test: `studyfrog-ui/tests/control_calendar_session.rs` (`calendar_session_add_creates_the_session_and_reminders_due_reports_it`).

Review (2026-11-22): all three criteria MET on the running window. Criterion 1: `calendar view week`, `timetable`, `month` and `day` each changed the screen (`cal-week.png`, `cal-timetable.png`, `cal-dec.png`, `cal-day.png`); `calendar period 2026-12-15` moved the month to December 2026 with the 15th selected (`cal-dec.png`). Criterion 2: a homework item due 2026-11-22 23:30 ("Soon test") was listed by `reminders due` on two consecutive calls (`{"lead_min":120,"title":"Soon test"}`), so the read does not mark it shown. Limit: the calendar-session half of the reminder was not exercised; no control command creates a calendar session, so `sessions` stayed empty. Criterion 3: `calendar homework 2026-11-25` answered `[{"due":"2026-11-25 17:00","title":"Essay","urgency":"Completed"}]`; `calendar view bogus` answers with the choices.

## Dependencies
EPIC-064, and the features named above.

## Open Questions
None.


Note (2026-11-22): moved back to review. The calendar-session reminder half is not checkable through the control protocol (no command creates a session), so the done move rested on the homework reminder evidence alone.

Review (2026-11-22, second): criterion 2 MET including the calendar-session half, now that `calendar session add` creates a session. In the running app (display :86): `ctl calendar session add "Epic64 check" --start "2026-11-22 22:42" --minutes 45` answered `{"session":{"id":"01M82P7043WAJX9M706JKS2KWT","minutes":45,"reminders":[15],"start":"2026-11-22 22:42","title":"Epic64 check"}}`; `ctl reminders due` at 22:32 answered `"sessions":[{"date":"2026-11-22","minutes_before":15,"title":"Epic64 check"}]` on two consecutive calls, so the read does not mark it shown. `ctl nav calendar` plus `ctl screenshot /tmp/claude-0/epic64/x.png` shows the session on 22 November in the month grid and in the day's sessions panel. Criteria 1 and 3 unchanged. Status set to done: all three criteria are ticked.
