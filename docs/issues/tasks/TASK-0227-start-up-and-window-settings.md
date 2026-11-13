---
id: TASK-0227
type: task
title: "Start-up and window settings"
status: done
epic: EPIC-027
parents: [STORY-0081]
estimate: 3
labels: [m11, ui, app]
acceptance_criteria:
  - "The start screen can be Home, Library or the screen left last"
  - "The sidebar can start folded or shown"
  - "The window's size and place (and whether it was maximised) can be remembered and restored, with a check that it lies on a screen that is there"
  - "All three are in the Appearance tab"
---

## Context
The owner asked on 2026-11-03 for a dark and a light mode and for UI settings in the application, and for an epic for them.

## Description
Settings for how the application starts.

## Acceptance Criteria
- [x] The start screen can be Home, Library or the screen left last
- [x] The sidebar can start folded or shown
- [x] The window's size and place (and whether it was maximised) can be remembered and restored, with a check that it lies on a screen that is there
- [x] All three are in the Appearance tab

## Design Notes
The window size goes through the viewport builder at start-up and is written when the window closes or resizes.

Built on 2026-11-03: start screen (home, library, last), sidebar folded at start and the window's size, place and maximised state remembered once it has settled; the saved window is read before the first frame.

## Dependencies
TASK-0221.

## Open Questions
None.
