---
id: BUG-0003
type: bug
title: "Create dialog header is crowded when the dialog is small"
status: done
epic: EPIC-025
parents: []
estimate: 2
labels: [m10, ui]
acceptance_criteria:
  - "At the normal width the stack, the type and the buttons do not overlap or truncate the type name"
  - "The dialog's size, More and Close controls sit on their own right-aligned group that never overlaps the pickers"
---

## Context
The owner tested 0.3.0 for a short while on 2026-11-03 and sent five screenshots with a list of observations: missing padding, a crowded Create dialog header, no confirmation when cancelling a filled dialog, no import suggestions, hard-to-read window buttons, no in-app exit button, a cramped streak and an overall dense UI.

## Description
In the normal-width Create dialog the type drop-down is squeezed to 'Ca' under the More and Enlarge buttons (screenshot 2); the enlarged dialog looks fine (screenshot 3).

## Acceptance Criteria
- [x] At the normal width the stack, the type and the buttons do not overlap or truncate the type name
- [x] The dialog's size, More and Close controls sit on their own right-aligned group that never overlaps the pickers

## Design Notes
None.

## Dependencies
None.

## Open Questions
None.
