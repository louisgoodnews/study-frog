---
id: TASK-0303
type: task
title: "Responsive centered Match tile grid with bounded text wrapping and honest empty state"
status: done
epic: EPIC-075
parents: ['STORY-0245']
estimate: 2
labels: [m14, ui]
acceptance_criteria:
  - 'The Match tile grid adapts column/row counts cleanly for 4, 6, or 8 pairs and centers across the available viewport width'
  - 'Tile text wraps cleanly inside tile bounds with ellipsis and full-text hover tooltip when truncated, never overflowing the tile border'
  - 'When fewer than 2 valid pairs exist in the chosen scope, an honest empty state card explains how to add cards or switch stacks'
---

## Context
Part of `STORY-0245`. Real flashcard fronts and answers are often longer than 3-word biology labels and need clean clipping/wrapping and an honest empty state.

## Description
Refine tile rendering in `crates/studyfrog-ui/src/view/match_grid.rs` to scale tiles responsively, clip/ellipsize text inside each tile rect with full-text hover tooltips, and render a calm redesign empty-state card when `< 2` pairs are available.

## Acceptance Criteria
- [x] The Match tile grid adapts column/row counts cleanly for 4, 6, or 8 pairs and centers across the available viewport width
- [x] Tile text wraps cleanly inside tile bounds with ellipsis and full-text hover tooltip when truncated, never overflowing the tile border
- [x] When fewer than 2 valid pairs exist in the chosen scope, an honest empty state card explains how to add cards or switch stacks

## Design Notes
- Keep `match_grid.rs` $\le 250$ LOC (P6).

## Dependencies
- `STORY-0245`, `TASK-0302`

## Open Questions
None.
