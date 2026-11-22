---
id: STORY-0197
type: story
title: "The item colour paints the item in rehearsal view"
status: done
epic: EPIC-065
parents: ['EPIC-065']
estimate: 3
labels: [m14, rehearsal, ui]
acceptance_criteria:
  - 'the item in the rehearsal view takes the item''s colour as a tint of its background, with the text at full contrast'
  - 'an item with no colour looks as it does now'
  - 'the tint passes the palette''s contrast test in both themes'
---

## Context
Items can be given a colour (item color, EPIC-032) that shows in the library and the editor, but the rehearsal view paints every item the same.

## Description
Use the tint helper the library uses for coloured rows; the palette test enforces the contrast rule.

## Acceptance Criteria
- [x] the item in the rehearsal view takes the item's colour as a tint of its background, with the text at full contrast
- [x] an item with no colour looks as it does now
- [x] the tint passes the palette's contrast test in both themes

## Design Notes
None.

## Implementation
- Already in place before this story: `item_look::card_colors` tints the rehearsal card's surface with the item's colour, lowering the share of the tint until the text passes the contrast floor; no colour leaves the surface as it was.
- Checked now: `tests/item_color_look.rs` (the floor, both themes) and `tests/item_color_ui.rs` (the colour on show and after the reveal) pass; a screenshot of the `run-long` demo with a teal item (light theme, 1280 by 800) shows the tint and readable text. The dark theme is covered by the same floor test, not by a screenshot.
- The rating-row spacer of STORY-0196 moved the layout; the item area's allowance was raised so the hints stay on screen.

## Dependencies
EPIC-065, and the features named above.

## Open Questions
None.
