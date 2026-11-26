---
id: STORY-0245
type: story
title: "Match mode scope picker, playable responsive tile grid, and empty state"
status: done
epic: EPIC-075
parents: ['EPIC-075']
estimate: 5
labels: [m14, ui]
acceptance_criteria:
  - 'Users can pick Match mode scope (entire vault or specific stacks), filter (All, Due first, Least familiar), and pair count (4, 6, 8 pairs)'
  - 'The HUD distinguishes Restart (reshuffle current round) from New round (deal fresh pairs) and fixes the Exit button tooltip'
  - 'The tile grid fills and centers within the viewport cleanly, wraps or truncates long card text inside tile bounds with full-text hover preview, and shows an honest empty state when fewer than 2 valid pairs exist'
  - 'Headless UI tests verify scope switching, round dealing, empty state, and accessible widget names'
---

## Context
Even once real vault cards are loaded (`STORY-0244`), Match mode needs scope selection, fresh-round dealing, readable tile sizing for real flashcard text, and a clear empty state instead of silent biology fallback.

## Description
Upgrade the Match screen (`crates/studyfrog-ui/src/view/match_grid.rs`, `match_victory.rs`, `match_ui.rs`):
- Compact scope, filter, and pair-count controls in the Match header.
- Distinct **Restart** and **New round** actions plus corrected **Exit** tooltip.
- Responsive centered grid with bounded wrapping/truncation and hover tooltips for long card text.
- Clear empty state when the selected scope has fewer than 2 valid cards/questions.

## Acceptance Criteria
- [x] Users can pick Match mode scope (entire vault or specific stacks), filter (All, Due first, Least familiar), and pair count (4, 6, 8 pairs)
- [x] The HUD distinguishes Restart (reshuffle current round) from New round (deal fresh pairs) and fixes the Exit button tooltip
- [x] The tile grid fills and centers within the viewport cleanly, wraps or truncates long card text inside tile bounds with full-text hover preview, and shows an honest empty state when fewer than 2 valid pairs exist
- [x] Headless UI tests verify scope switching, round dealing, empty state, and accessible widget names

## Design Notes
- Use only `palette_of(ui)` / `style_of` tokens and keep every file $\le 250$ LOC (Invariant P6).

## Dependencies
- `EPIC-075`, `STORY-0244`
- Child tasks: `TASK-0302`, `TASK-0303`, `TASK-0304`

## Open Questions
None.
