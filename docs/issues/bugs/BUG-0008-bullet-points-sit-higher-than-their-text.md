---
id: BUG-0008
type: bug
title: Bullet points sit higher than their text
status: done
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
- [x] In the preview the bullet, the number or the check box is level with the first line of its item, at every font size and for wrapped lines

## Design Notes
`view/preview.rs` `list_item` draws the marker as a separate label in `horizontal_top` next to a vertical block with its own spacing and a different glyph size, so the two baselines differ. Draw the marker in the first line's row or give it the same font size and the item's top spacing. Check with a screenshot (`--screenshot`) and a layout test on the marker and the first line.

Built on 2026-11-03: the marker is now drawn in a row as tall as a paragraph's first line, with the same vertical centring (`view/preview.rs`). The cause was exactly that: paragraphs sit in rows at least as tall as a button with their text centred, the marker sat at the top. A layout test compares the marker's centre with its text's for bullets and numbers; it fails by 2.5 points on the old code.

## Dependencies
None.

## Open Questions
None.
