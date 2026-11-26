---
id: BUG-0059
type: bug
title: "Item view cuts off Title input, Color hex field, and under-sizes headline and Front/Back editors"
status: backlog
estimate: 2
labels: [m14, ui]
acceptance_criteria:
  - "The Item view's gradient headline panel spans the full width of the editor column cleanly without stopping mid-way"
  - "The Title input fills the available editor width (and scrolls horizontally for very long titles) instead of cutting off mid-sentence"
  - "The Front and Back text editors fill the available width of their enclosing cards (or align cleanly with the reading measure without leaving mismatched empty space inside wide cards)"
  - "The Color hex input ('#RRGGBB') in Details > Properties is wide enough to display a full 7-character hex code without clipping to '#RRG...'"
  - "Headless UI tests verify these fields at 1280x800 and 960x640 with long titles and full hex values"
---

## Context
Reported by the user with a screenshot of the Item view (card editor for *"Was versteht man unter dem nomothetischen Ansatz in der Diagnostik?"*):
1. **Title input cut off:** The `Title` `TextEdit` stops around ~380 px wide, cutting off `"Was versteht man unter dem nomothetischen"` mid-sentence even though plenty of horizontal space is free to the right.
2. **Gradient headline under-sized:** The gradient headline banner matches only ~45% of the editor column width instead of spanning the column width like the `Front` / `Back` cards below it.
3. **Front & Back editors half-width inside full-width cards:** The `Front` and `Back` card frames and their formatting toolbars span the full editor column, while the dark `TextEdit` boxes inside them only occupy the left half of each card.
4. **Details > Properties > Color hex input cut off:** The hex color input next to the swatches is clipped to `#RRG...`, making it impossible to read or verify a `#RRGGBB` value.

## Description
Fix the width sizing and cut-off bugs in the Item view (`crates/studyfrog-ui/src/view/editor.rs`, `crates/studyfrog-ui/src/view/editor_md.rs`, and `crates/studyfrog-ui/src/view/item_details/tab_properties.rs`) so the gradient headline, `Title` input, `Front`/`Back` editors, and Details `Color` hex input use their available width cleanly without clipping.

## Acceptance Criteria
- [ ] The Item view's gradient headline panel spans the full width of the editor column cleanly without stopping mid-way
- [ ] The Title input fills the available editor width (and scrolls horizontally for very long titles) instead of cutting off mid-sentence
- [ ] The Front and Back text editors fill the available width of their enclosing cards (or align cleanly with the reading measure without leaving mismatched empty space inside wide cards)
- [ ] The Color hex input (`#RRGGBB`) in Details > Properties is wide enough to display a full 7-character hex code without clipping to `#RRG...`
- [ ] Headless UI tests verify these fields at 1280x800 and 960x640 with long titles and full hex values

## Design Notes
- Likely root cause: reading-measure width clamping (`STORY-0114`) applied to inner `TextEdit` / `gradient_panel` widgets while their enclosing `Front`/`Back` cards expand to `ui.available_width()`, plus an overly narrow `desired_width` on the Color `#RRGGBB` input in `tab_properties.rs`.
- Follow Invariant P6 (files $\le 250$ LOC) and preserve accessible widget labels.

## Dependencies
- Related: `STORY-0243` (horizontal scrollability and responsive sizing for constrained widgets)

## Open Questions
None.
