---
id: BUG-0008
type: bug
title: Bullet points sit higher than their text
status: backlog
epic: EPIC-026
parents: [STORY-0074]
estimate: 2
labels: [m11, ui]
acceptance_criteria:
  - "In the preview the bullet, the number or the check box is level with the first line of its item, at every font size and for wrapped lines"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports.

## Description
In the preview and the rendered notes the list marker is higher than the text next to it (screenshot of the owner).

## Acceptance Criteria
- [ ] In the preview the bullet, the number or the check box is level with the first line of its item, at every font size and for wrapped lines

## Design Notes
`view/preview.rs` `list_item` draws the marker as a separate label in `horizontal_top` next to a vertical block with its own spacing and a different glyph size, so the two baselines differ. Draw the marker in the first line's row or give it the same font size and the item's top spacing. Check with a screenshot (`--screenshot`) and a layout test on the marker and the first line.

## Dependencies
None.

## Open Questions
None.
