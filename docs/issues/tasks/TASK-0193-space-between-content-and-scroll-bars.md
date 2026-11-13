---
id: TASK-0193
type: task
title: "Space between content and scroll bars"
status: done
epic: EPIC-025
parents: [STORY-0072]
estimate: 1
labels: [m10, ui]
acceptance_criteria:
  - "A scroll bar no longer overlaps the content next to it, in every scroll area, in both themes"
  - "The reserved width is set once in the theme"
---

## Context
The owner asked on 2026-11-03, after trying the 0.3.1 build: widgets next to a scroll bar need padding, and the rehearsal view needs a progress bar against the items still open.

## Description
Scroll bars floated over the content, so widgets and table edges ran under them.

## Acceptance Criteria
- [x] A scroll bar no longer overlaps the content next to it, in every scroll area, in both themes
- [x] The reserved width is set once in the theme

## Design Notes
`theme::apply` sets `spacing.scroll.floating_allocated_width` to 12 and `bar_outer_margin` to 4, so a floating bar reserves its own width everywhere. Checked on the stack page and help in screenshots and by a test of the style.

## Dependencies
None.

## Open Questions
None.
