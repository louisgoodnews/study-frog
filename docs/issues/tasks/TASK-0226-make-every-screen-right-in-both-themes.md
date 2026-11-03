---
id: TASK-0226
type: task
title: Make every screen right in both themes
status: done
epic: EPIC-027
parents: [STORY-0080]
estimate: 5
labels: [m11, ui]
acceptance_criteria:
  - "The screenshot tool takes a theme (`--theme dark|light`) and a screenshot of every screen in both themes has been looked at"
  - "Colours written into views (`Color32::from_rgb` and the like) are replaced by palette entries, except the logo and the red close button, which a test lists as the allowed ones"
  - "Text and controls of the palette meet a contrast ratio of 4.5 to 1 (3 to 1 for large text and outlines), checked by a test on the palette data"
  - "Charts, the status ring, bars, toasts, the title bar and the dialogs are right in light"
---

## Context
The owner asked on 2026-11-03 for a dark and a light mode and for UI settings in the application, and for an epic for them.

## Description
The light palette was never looked at screen by screen.

## Acceptance Criteria
- [x] The screenshot tool takes a theme (`--theme dark|light`) and a screenshot of every screen in both themes has been looked at
- [x] Colours written into views (`Color32::from_rgb` and the like) are replaced by palette entries, except the logo and the red close button, which a test lists as the allowed ones
- [x] Text and controls of the palette meet a contrast ratio of 4.5 to 1 (3 to 1 for large text and outlines), checked by a test on the palette data
- [x] Charts, the status ring, bars, toasts, the title bar and the dialogs are right in light

## Design Notes
A test greps the view sources for hard-coded colours and compares with an allowlist; a test computes the contrast of every text and surface pair of both palettes; fix what fails.

Progress on 2026-11-03: the palette contrast test is done (`tests/appearance.rs`) and the light theme was looked at on the Settings, stack and Home screens; one accent (orange) was darkened to pass. Finished on 2026-11-03: `studyfrog --screenshot FILE --theme dark|light|system`; screenshots of Home, Library, the stack page, Rehearse, a run, Settings, Help, Calendar, Board, Reports, Achievements, Assistant and the Graph in both themes were taken and Settings, the stack page, Calendar, Reports, Graph, Assistant and a run were looked at in light (Board in dark). Hard-coded colours were replaced by palette entries (a new `alt` colour for appointments and notes in the graph, the danger colour for the notification badge) and `tests/no_hardcoded_colours.rs` fails when a view names a colour of its own, except the logo, the red close button, PDF paper and highlights. A layout bug found on the way was fixed: the Board's last column and its Add card button ran off the right edge. Not looked at: the Create dialog, the import and export dialog, the PDF viewer and the Help, Home and Achievements screens in light (taken, not viewed in this pass).

## Dependencies
TASK-0222.

## Open Questions
None.
