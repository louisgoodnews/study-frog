---
id: EPIC-065
type: epic
title: "Rehearsal view: item size, previous ratings and item colour"
status: backlog
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
- [ ] The rehearsal view gives the item more of the screen, and its colour and previous rating show clearly
- [ ] The rehearsal view's controls keep what they do; only their layout and colours change

## Design Notes
Reuse the palette tokens and the density scale; no hard-coded colours (tests/no_hardcoded_colours.rs).

## Dependencies
The rehearsal view (view/run*.rs).

## Open Questions
None.
