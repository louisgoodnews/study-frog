---
id: TASK-0226
type: task
title: Make every screen right in both themes
status: in_progress
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
- [ ] The screenshot tool takes a theme (`--theme dark|light`) and a screenshot of every screen in both themes has been looked at
- [ ] Colours written into views (`Color32::from_rgb` and the like) are replaced by palette entries, except the logo and the red close button, which a test lists as the allowed ones
- [x] Text and controls of the palette meet a contrast ratio of 4.5 to 1 (3 to 1 for large text and outlines), checked by a test on the palette data
- [ ] Charts, the status ring, bars, toasts, the title bar and the dialogs are right in light

## Design Notes
A test greps the view sources for hard-coded colours and compares with an allowlist; a test computes the contrast of every text and surface pair of both palettes; fix what fails.

Progress on 2026-11-03: the palette contrast test is done (`tests/appearance.rs`) and the light theme was looked at on the Settings, stack and Home screens; one accent (orange) was darkened to pass. Still to do: a `--theme` option for the screenshot tool (a saved setting works meanwhile), a screenshot of every screen in both themes, replacing hard-coded colours in the views and the test that lists the allowed ones.

## Dependencies
TASK-0222.

## Open Questions
None.
