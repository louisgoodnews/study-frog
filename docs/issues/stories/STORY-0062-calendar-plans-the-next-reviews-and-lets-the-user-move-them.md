---
id: STORY-0062
type: story
title: Calendar plans the next reviews and lets the user move them
status: backlog
epic: EPIC-023
parents: [EPIC-023]
estimate: 8
labels: [m10, calendar, ui, core]
acceptance_criteria:
  - "After a rehearsal, the items' next reviews from the spaced-repetition schedule appear on the calendar, grouped by stack and day (for example '4 of 10 items of stack X on 18 Nov')"
  - "An entry can be dragged to another day, which reschedules its items, and the move can be undone"
  - "Complete and On hold stacks and items are not planned, and planning resumes if the status changes back"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
When a rehearsal ends the schedule already moves each item's next due date. Show those dates on the calendar as appointments and let the user move them to days that suit them.

## Acceptance Criteria
- [ ] After a rehearsal, the items' next reviews from the spaced-repetition schedule appear on the calendar, grouped by stack and day (for example '4 of 10 items of stack X on 18 Nov')
- [ ] An entry can be dragged to another day, which reschedules its items, and the move can be undone
- [ ] Complete and On hold stacks and items are not planned, and planning resumes if the status changes back

## Design Notes
The owner's example (2026-11-02): 4 of the 10 items of stack X fall due on 18 Nov, but the user is away that day, so they drag the entry to 20 Nov. A pure planner in the calendar crate turns the schedules and statuses into entries, one per stack and day, with the item count and the stack's size. Entries are computed from the schedule, not stored, so a finished rehearsal updates the calendar at once and manual sessions are never touched. Dragging an entry moves the due date of its items to the new day; the memory state (stability and difficulty) stays untouched, so the next rehearsal of an item replaces the moved date with the algorithm's own, as after any review. Moves are undoable and each shows a notice. Keyboard users move an entry with a date picker in its menu (drag alone is not accessible). A stack's own user-set due date still shows as before. Items that are Complete or On hold (STORY-0061) are skipped.

Tasks: TASK-0146, TASK-0147, TASK-0148.

## Dependencies
STORY-0061 (statuses) for skipping finished items; the rest stands on the existing schedule.

## Open Questions
Answered by the owner on 2026-11-02: a moved item stays out of the rehearsal queue until its new day, and a move that lands after an exam or due date of the stack shows a warning, not a block.
