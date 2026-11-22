---
id: BUG-0044
type: bug
title: "Calendar Timetable view lists no lectures after the app restarts, though the lecture is stored and shown in the week view"
status: done
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

## Fix (2026-11-22)

Cause: the Timetable view draws `planner.timetable`. That list was filled only by `PlanMsg::TimetableChanged`, which follows a save. `PlanMsg::TimetableLoaded` was matched in `planner.rs` but never sent. The week view reads the same table through `load_sessions`, which is why it showed the lectures.

Fix: `exec_plan::load_sessions` now answers with the sessions and then `TimetableLoaded`, so opening the calendar loads the stored lectures. The homework load does the same (`exec_homework::load`). Test: `studyfrog-ui/tests/calendar_timetable.rs::stored_lectures_are_listed_after_a_restart`. Before the fix it failed with `left: []`; it passes now.

Verified in the running app (2026-11-22, a copy of the built binary on display :88, vault opened after a restart with three lecture rows stored and no save): the Timetable view lists Monday 09:00 Statistik-Vorlesung, Monday 13:00 Englisch-Seminar and Tuesday 10:00 Statistik-Uebung.
- Before: `/tmp/claude-0/review-a/tt-restart.png` (no lectures).
- After, first restart at 960 by 640: `/tmp/claude-0/homework/a-timetable-restart.png`.
- After, second restart at 1280 by 800: `/tmp/claude-0/homework/h-timetable-after-second-restart-1280.png`.
- After, final build at 960 by 640: `/tmp/claude-0/homework/i-timetable-final-960.png`.
