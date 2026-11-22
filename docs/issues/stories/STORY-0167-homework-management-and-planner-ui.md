---
id: STORY-0167
type: story
title: "Interactive Homework Planner Screen and Creation Dialog"
status: done
epic: EPIC-056
parents: ['EPIC-056']
estimate: 5
labels: [m14, ui]
acceptance_criteria:
  - 'Dedicated Homework navigation tab/view accessible from main navigation rail'
  - 'Interactive assignment list with status filtering (All, Active, Due Today, Overdue, Completed)'
  - 'Modal dialog to create, edit, estimate, and complete homework tasks with course association'
---

## Context
Students require a frictionless and focused interface to manage their assignments, view upcoming tasks, and tick them off as completed.

## Description
Build the Homework management user interface in `studyfrog-ui`:
- Add a Homework view with clear visual hierarchy, subject badges, and due date relative countdowns.
- Provide quick-filter tabs for Active, Due Today, Overdue, and Completed assignments.
- Implement an accessible modal creation/edit dialog with subject selection, date-time picker, estimated duration slider, and reminder lead times.

## Acceptance Criteria
- [x] Dedicated Homework navigation tab/view accessible from main navigation rail
- [x] Interactive assignment list with status filtering (All, Active, Due Today, Overdue, Completed)
- [x] Modal dialog to create, edit, estimate, and complete homework tasks with course association

## Design Notes
Adhere strictly to StudyFrog design tokens, accessible names, keyboard navigation, and `palette_of(ui)` theming (no raw RGB).

## Dependencies
EPIC-056

## Implementation (2026-11-21)
Homework screen in the Plan group of the sidebar and the burger menu (`Screen::Homework`, `view/homework.rs`). Tabs All, Active, Due today, Overdue and Completed (`studyfrog-calendar::homework_query::{HomeworkTab, tab_filter}`). Each row shows urgency, a countdown (`countdown`), the stack as a course badge, priority, estimate, and checklist steps to tick. The dialog (`homework_form.rs`, `view/homework_form_view.rs`) creates and edits items with title, stack, due date and time, an estimate slider, priority, a checklist and notes. Start, Complete and Reopen move an item on through `next_status`. Saves go through `save_homework_checked`; the screen is `homework_update.rs` (pure) with `exec_homework.rs` as the worker side. Tests: `studyfrog-calendar/tests/homework_query.rs`, `studyfrog-ui/tests/homework_screen.rs`.

Deleting an item asks for confirmation in the dialog first ("Delete item", then "Delete it" or "Keep it"), and the dialog closes once the item is gone.

Not done: reminder lead-time choice in the dialog (lead times are global), and timetable integration beyond the stack course. The screen has not been exercised in the running app.

## Window check (2026-11-22)

Checked in the running window: a homework item is created, edited and deleted through "Delete item", "Delete it" removes it and closes the dialog. Fixed in this check: at 960 by 640 the dialog ran past the window and its Delete row could not be reached; the dialog now scrolls.

## Implementation (2026-11-22, BUG-0043)

The homework screen now has a Subject row (`HomeworkUi.subject`, kept by the status tabs; `homework subject <stack folder>|all` in the control protocol) and lists the week's timetable beside the items, with the lessons on each item's due day (`studyfrog_calendar::timetable::lessons_note`, `view/calendar_timetable.rs::timetable_panel`). Tests: `studyfrog-ui/tests/homework_screen.rs`, `control_homework.rs`, `calendar_timetable.rs`; `studyfrog-calendar/tests/homework_query.rs`, `timetable.rs`. Checked in the running app at 1280 by 800 and 960 by 640 (`/tmp/claude-0/homework/k-hw-final-1280-all.png`, `j-hw-final-960-subject-tab.png`). Timetable reads on the homework screen came with BUG-0044.

## Open Questions
None.
