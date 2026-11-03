---
id: TASK-0190
type: task
title: Readable window buttons
status: blocked
epic: EPIC-025
parents: [STORY-0072]
estimate: 3
labels: [m10, ui, app]
acceptance_criteria:
  - "Minimize, maximize and close are clearly visible against the dark and the light theme"
  - "They work on X11 and Wayland, and double-click on the bar maximizes"
---

## Context
The owner tested 0.3.0 for a short while on 2026-11-03 and sent five screenshots with a list of observations: missing padding, a crowded Create dialog header, no confirmation when cancelling a filled dialog, no import suggestions, hard-to-read window buttons, no in-app exit button, a cramped streak and an overall dense UI.

## Description
The window manager's symbols are hard to see. Draw StudyFrog's own title bar with large buttons or give the system ones a contrast fix.

## Acceptance Criteria
- [ ] Minimize, maximize and close are clearly visible against the dark and the light theme
- [ ] They work on X11 and Wayland, and double-click on the bar maximizes

## Design Notes
Compare drawing our own title bar (works the same everywhere, we must handle drag, resize edges and snapping) with keeping the system bar and only choosing a better theme hint. Decide with the owner.

## Dependencies
Blocked on the owner's decision between drawing StudyFrog's own title bar and keeping the system's. A custom bar means taking over dragging, resizing at the edges, snapping and double-click maximize on X11 and Wayland, which cannot be checked without a real desktop session; the session this was built in has none.

## Open Questions
None.
