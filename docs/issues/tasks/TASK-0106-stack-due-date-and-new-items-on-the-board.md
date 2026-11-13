---
id: TASK-0106
type: task
title: "Stack due date and new items on the board"
status: done
epic: EPIC-020
parents: [STORY-0039]
estimate: 3
labels: [m8, ui]
acceptance_criteria:
  - "The create dialog takes an optional due date and reminders for a stack and puts a deadline on the calendar"
  - "Every item made in the dialog gets a card in the first board column, linked to its stack"
  - "A failed follow-up is reported without undoing the item"
---

## Context
The owner found the setup convoluted, the home page and calendar mostly empty, and asked for evidence-based UX, calendar entries with reminders, stack due dates and new items on the board.

## Description
Stack due date and new items on the board.

## Acceptance Criteria
- [x] The create dialog takes an optional due date and reminders for a stack and puts a deadline on the calendar
- [x] Every item made in the dialog gets a card in the first board column, linked to its stack
- [x] A failed follow-up is reported without undoing the item

## Design Notes
service::track::create_tracked.

## Dependencies
None.

## Open Questions
None.
