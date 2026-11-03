---
id: BUG-0002
type: bug
title: The stack page has no padding at its left edge
status: done
epic: EPIC-025
parents: []
estimate: 2
labels: [m10, ui]
acceptance_criteria:
  - "The content of the stack page has the same left and top margin as the other screens, at every window width"
  - "The Summary cards and rows keep a gap to the right edge and the scroll bar"
---

## Context
The owner tested 0.3.0 for a short while on 2026-11-03 and sent five screenshots with a list of observations: missing padding, a crowded Create dialog header, no confirmation when cancelling a filled dialog, no import suggestions, hard-to-read window buttons, no in-app exit button, a cramped streak and an overall dense UI.

## Description
The stack page (breadcrumb, title, tabs and Summary cards) touches the stack list on its left (screenshot 1).

## Acceptance Criteria
- [x] The content of the stack page has the same left and top margin as the other screens, at every window width
- [x] The Summary cards and rows keep a gap to the right edge and the scroll bar

## Design Notes
None.

## Dependencies
None.

## Open Questions
None.
