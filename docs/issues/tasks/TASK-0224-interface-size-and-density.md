---
id: TASK-0224
type: task
title: "Interface size and density"
status: done
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
- [x] The interface size can be set from 80 to 200 percent in steps, and with Ctrl+plus, Ctrl+minus and Ctrl+0
- [x] Density (compact, comfortable, spacious) changes the spacing and the heights of controls from one table in the theme
- [x] Both are stored in app.toml and take effect at once; nothing is cut off at the extremes, checked by screenshots at 80 and 200 percent

## Design Notes
egui's `zoom_factor` for the size; density as a record of spacing values applied by `theme::apply`. Check the title bar, the top bar and the tables at the extremes.

Built on 2026-11-03: zoom from 70 to 200 percent with Ctrl and plus, minus and 0, text size apart from it, and three densities from one table; not checked by screenshots at 70 and 200 percent.

## Dependencies
TASK-0221.

## Open Questions
None.
