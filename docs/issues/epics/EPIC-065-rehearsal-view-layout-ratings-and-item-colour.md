---
id: EPIC-065
type: epic
title: "Rehearsal view: item size, previous ratings and item colour"
status: done
parents: []
labels: [m14, rehearsal, ui, ux]
acceptance_criteria:
  - 'The rehearsal view gives the item more of the screen, and its colour and previous rating show clearly'
  - 'The rehearsal view''s controls keep what they do; only their layout and colours change'
---

## Context
The rehearsal view gives the item little room, shows no sign of how the item was rated last time, and does not use the item's colour. STORY-0170 widened the spacing, which makes the crowding worse at small sizes.

## Description
Three stories: the item takes more of the screen; the rating buttons show the previous rating in its colour; the item's colour paints its display.

## Acceptance Criteria
- [x] The rehearsal view gives the item more of the screen, and its colour and previous rating show clearly
- [x] The rehearsal view's controls keep what they do; only their layout and colours change

## Design Notes
Reuse the palette tokens and the density scale; no hard-coded colours (tests/no_hardcoded_colours.rs).

## Dependencies
The rehearsal view (view/run*.rs).

## Open Questions
None.

## Verification (2026-11-22)
Checked in the running app at 1280 by 800, 1600 by 1000 and the 640 by 420 minimum (`/tmp/claude-0/epic65/`):
- Item and colour: `fix6-prev-hard-card1-1280x800.png` and `fix6-prev-hard-card1-1600x1000.png` (teal item, dark theme), `fix6-long-item-prev-again-red-1280x800.png` (long item scrolls, buttons in view).
- Previous rating: `fix6-prev-hard-card1-1280x800.png` (orange, Medium), `fix6-long-item-prev-again-red-1280x800.png` (red, Difficult), `prev-good-card1-1280x800.png` (green, Easy, before the label fix), `light-prev-teal-card1-1280x800.png` (light theme, green).
- Controls, each clicked and checked in the app state: Show answer, Skip, Hard, the Medium mark, Comments, Pause and resume, Edit (opens the edit dialog and pauses the run), Stop.
- Known at the minimum: the header and the top strip overlap and the hint line is cut; see BUG-0021.

