---
id: BUG-0043
type: bug
title: "Homework planner shows no timetable and has no subject filter"
status: backlog
epic: EPIC-056
parents: ['EPIC-056']
estimate: 3
labels: [m14, ui, calendar]
acceptance_criteria:
  - "The homework planner shows the student's timetable beside the due items, so a due date can be placed against the lessons of that day"
  - "The planner can be filtered by subject (stack), and the filter is kept while the tabs change"
---

## Context
Found by the window audit of EPIC-056 (2026-11-22, display :95). Both criteria of the epic that are not met are about the timetable and the subject filter. Findings table: /tmp/claude-0/audit/g5/findings.md.

## Description
The planner lists items by tab (All, Active, Due today, Overdue, Completed) with no timetable beside them and no way to narrow the list to one subject.

## Acceptance Criteria
- The homework planner shows the student's timetable beside the due items, so a due date can be placed against the lessons of that day
- The planner can be filtered by subject (stack), and the filter is kept while the tabs change

## Design Notes
Reuse the calendar's timetable data; the subject is the homework item's stack, as the course field already holds it.

## Dependencies
- EPIC-056
- STORY-0167

## Open Questions
- None.

## Verification (2026-11-22)

Reproduced in the running app (head build copy, display :92). Both criteria are NOT MET, so the bug stays open (status backlog).
- Criterion 1 (timetable beside the due items): NOT MET. The Homework screen shows the two homework cards, the tabs and no timetable: `/tmp/claude-0/verifyA/03-homework.png`. The Calendar Timetable view listed no lectures after a restart, although a lecture was stored; this is BUG-0044. The week view shows the lecture and the deadline markers together (`/tmp/claude-0/verifyA/11-week.png`), but that is not the planner.
- Criterion 2 (subject filter kept across tabs): NOT MET. The Homework screen has no subject control (`/tmp/claude-0/verifyA/03-homework.png`).
