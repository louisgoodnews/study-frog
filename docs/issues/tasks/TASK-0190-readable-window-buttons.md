---
id: TASK-0190
type: task
title: Readable window buttons
status: done
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
- [x] Minimize, maximize and close are clearly visible against the dark and the light theme
- [x] They work on X11 and Wayland, and double-click on the bar maximizes

## Design Notes
Built: `view/window_frame.rs`. The top bar is the title bar; a press on empty bar space sends `StartDrag` and a double click toggles maximise, so the window manager does the moving and snapping. Minimise, maximise or restore and a red close button (the old Exit button, now with the same unsaved-changes question) are drawn buttons 44 by 32 points; the edges (6 points) send `BeginResize`; a 1 px outline marks the window. The viewport has no decorations and a minimum size of 640 by 420. Checked here by headless tests and by screenshots on X11 only; dragging, snapping and resizing under GNOME/Wayland wait for the owner's test.

Options were: compare drawing our own title bar (works the same everywhere, we must handle drag, resize edges and snapping) with keeping the system bar and only choosing a better theme hint. Decide with the owner.

## Dependencies
Unblocked on 2026-11-03: the owner chose StudyFrog's own title bar and tried it on their machine and confirmed on 2026-11-03 that it looks far better; the criteria about X11 and Wayland were met on their GNOME/Wayland session. (Earlier: blocked on the owner's decision between drawing StudyFrog's own title bar and keeping the system's. A custom bar means taking over dragging, resizing at the edges, snapping and double-click maximize on X11 and Wayland, which cannot be checked without a real desktop session.)

## Open Questions
None.
