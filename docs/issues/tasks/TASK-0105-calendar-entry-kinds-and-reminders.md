---
id: TASK-0105
type: task
title: Calendar entry kinds and reminders
status: done
epic: EPIC-020
parents: [STORY-0039]
estimate: 5
labels: [m8, ui]
acceptance_criteria:
  - "An entry is a study session, task, appointment, exam or deadline with up to five reminders"
  - "Each reminder shows once as a toast and a notification while the app is open"
  - "The month grid fills the window with coloured chips and a coming-up list sits beside it"
---

## Context
The owner found the setup convoluted, the home page and calendar mostly empty, and asked for evidence-based UX, calendar entries with reminders, stack due dates and new items on the board.

## Description
Calendar entry kinds and reminders.

## Acceptance Criteria
- [x] An entry is a study session, task, appointment, exam or deadline with up to five reminders
- [x] Each reminder shows once as a toast and a notification while the app is open
- [x] The month grid fills the window with coloured chips and a coming-up list sits beside it

## Design Notes
Schema v8 (session.kind, session_reminder, reminder_fired); calendar::reminders is pure; the UI checks every 30 ticks.

## Dependencies
None.

## Open Questions
None.
