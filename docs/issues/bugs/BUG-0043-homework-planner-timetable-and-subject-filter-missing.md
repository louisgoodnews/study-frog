---
id: BUG-0043
type: bug
title: "Homework planner shows no timetable and has no subject filter"
status: done
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

## Review (2026-11-22)

Review (2026-11-22): still reproduced in the running app (head copy, display :81). Criterion 1 NOT MET: the Homework screen shows the cards, the tabs and no timetable (/tmp/claude-0/review-a/hw-1280.png). Criterion 2 NOT MET: no subject control on that screen (/tmp/claude-0/review-a/hw-1280.png). Status stays backlog.

## Fix (2026-11-22)

Built on the homework screen (`crates/studyfrog-ui/src/view/homework.rs`, `view/calendar_timetable.rs`):
- Timetable beside the due items: the week's timetable is listed beside the list (above it in a window narrower than 640 px). Each item also names the lessons on its due day, for example "Monday: 09:00 Statistik-Vorlesung, 13:00 Englisch-Seminar" or "Wednesday: no lessons" (`studyfrog_calendar::timetable::lessons_note`). The timetable is loaded with the homework (BUG-0044).
- Subject filter: a Subject row ("All" and one chip per stack that homework belongs to). The choice is `HomeworkUi.subject`, and a status tab change keeps it. Control: `homework subject <stack folder>|all`. The subject of an item is its stack, as in the course field.

Tests: `studyfrog-calendar/tests/timetable.rs` (`lessons_on_a_day_are_that_weekdays_entries_in_start_order`, `lessons_note_names_the_day_and_its_lessons`), `homework_query.rs` (`subjects_list_each_stack_once_and_skip_items_without_one`), `studyfrog-ui/tests/homework_screen.rs` (`the_subject_filter_is_kept_when_the_tab_changes`, `the_subject_filter_narrows_the_tab_list_to_that_stack`), `control_homework.rs` (`a_subject_is_a_stack_folder_or_all_and_reading_it_is_allowed`, `an_unknown_subject_folder_is_refused_before_anything_is_done`), `calendar_timetable.rs` (`the_homework_screen_gets_the_timetable_beside_its_items`).

Verified in the running app (2026-11-22, a copy of the built binary on display :88, vault with two stacks, three homework items and three lectures):
- Before: `/tmp/claude-0/review-a/hw-1280.png` (no timetable, no subject control).
- After, all subjects at 1280 by 800: `/tmp/claude-0/homework/k-hw-final-1280-all.png`. The timetable sits beside the list, and "Essay Englisch" due Wednesday shows "Wednesday: no lessons".
- After, Statistik chosen and the Active tab at 960 by 640: `/tmp/claude-0/homework/j-hw-final-960-subject-tab.png`. Only the Statistik item is listed, so the filter is kept across the tab change.
