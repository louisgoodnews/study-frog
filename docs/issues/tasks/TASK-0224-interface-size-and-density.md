---
id: TASK-0224
type: task
title: Interface size and density
status: backlog
epic: EPIC-027
parents: [STORY-0081]
estimate: 5
labels: [m11, ui]
acceptance_criteria:
  - "The interface size can be set from 80 to 200 percent in steps, and with Ctrl+plus, Ctrl+minus and Ctrl+0"
  - "Density (compact, comfortable, spacious) changes the spacing and the heights of controls from one table in the theme"
  - "Both are stored in app.toml and take effect at once; nothing is cut off at the extremes, checked by screenshots at 80 and 200 percent"
---

## Context
The owner asked on 2026-11-03 for a dark and a light mode and for UI settings in the application, and for an epic for them.

## Description
Make the application usable on small screens and for people who need larger text.

## Acceptance Criteria
- [ ] The interface size can be set from 80 to 200 percent in steps, and with Ctrl+plus, Ctrl+minus and Ctrl+0
- [ ] Density (compact, comfortable, spacious) changes the spacing and the heights of controls from one table in the theme
- [ ] Both are stored in app.toml and take effect at once; nothing is cut off at the extremes, checked by screenshots at 80 and 200 percent

## Design Notes
egui's `zoom_factor` for the size; density as a record of spacing values applied by `theme::apply`. Check the title bar, the top bar and the tables at the extremes.

## Dependencies
TASK-0221.

## Open Questions
None.
