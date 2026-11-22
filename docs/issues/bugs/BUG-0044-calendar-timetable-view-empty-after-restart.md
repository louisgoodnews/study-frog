---
id: BUG-0044
type: bug
title: "Calendar Timetable view lists no lectures after the app restarts, though the lecture is stored and shown in the week view"
status: backlog
epic: EPIC-056
parents: ['EPIC-056']
estimate: 1
labels: [m14, calendar, ui]
acceptance_criteria:
  - "After a restart, the Calendar Timetable view lists every stored lecture or class on its weekday, as the week view does"
  - "Stored lectures are listed without first saving another timetable entry"
---

## Context
Found while checking BUG-0043 (the homework planner, 2026-11-22). A lecture "Statistik-Vorlesung" (Monday 09:00, linked to the Statistik stack) was stored in the vault index. After the app restarted and the vault was reopened, the Calendar Timetable view showed "No scheduled lectures or lessons" for every weekday (`/tmp/claude-0/verifyA/10-timetable-restart.png`). The week view of the same week shows the lecture (`/tmp/claude-0/verifyA/11-week.png`).

## Description
The Timetable view draws `state.planner.timetable`. That list is filled only by `PlanMsg::TimetableChanged`, which follows a save. `PlanMsg::TimetableLoaded` is matched in `crates/studyfrog-ui/src/planner.rs` but is never sent, so the stored entries are not loaded when the calendar opens. The week and month views get the same entries through `load_sessions` in `crates/studyfrog-ui/src/exec_plan.rs`, which is why they show them.

Expected fix: load the timetable when the calendar opens, as the sessions load does.

Note on the reproduction: the lecture row was written into the vault index directly, because the add form would not take typed text in the test environment. The save path through the form was not exercised.

## Acceptance Criteria
- After a restart, the Calendar Timetable view lists every stored lecture or class on its weekday, as the week view does
- Stored lectures are listed without first saving another timetable entry

## Design Notes
Keep the loading in pure free functions and within the file-size limit (P6). Reuse the load path the sessions already use rather than adding a second query.

## Dependencies
- EPIC-056 (calendar and timetable integration)
- STORY-0141 (timetable entries)

## Open Questions
- None.

## Review (2026-11-22)

Review (2026-11-22): still reproduced. After the app restarts and the vault reopens, the Calendar Timetable view lists no lectures on any weekday (/tmp/claude-0/review-a/tt-restart.png), while the week view shows the same lecture on Monday 09:00 (/tmp/claude-0/review-a/week-next.png). Both criteria NOT MET. The lecture was added as a row in the vault index, because the Add lecture form takes no typed text in this environment. Status stays backlog.
