---
id: STORY-0197
type: story
title: "The item colour paints the item in rehearsal view"
status: backlog
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
- [ ] the item in the rehearsal view takes the item's colour as a tint of its background, with the text at full contrast
- [ ] an item with no colour looks as it does now
- [ ] the tint passes the palette's contrast test in both themes

## Design Notes
None.

## Dependencies
EPIC-065, and the features named above.

## Open Questions
None.
