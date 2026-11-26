---
id: TASK-0302
type: task
title: "Match mode scope selector, pair-count picker, and Restart vs New Round HUD"
status: done
epic: EPIC-075
parents: ['STORY-0245']
estimate: 2
labels: [m14, ui]
acceptance_criteria:
  - 'Match HUD/setup bar lets the user select Entire vault or a specific stack, filter (All, Due first, Least familiar), and pair count (4, 6, 8 pairs)'
  - 'Restart reshuffles the current round while New round deals a fresh subset of pairs from the deck'
  - 'The Exit button tooltip accurately reads "Leaves Match mode." instead of "Quits the app."'
---

## Context
Part of `STORY-0245`. Gives users control over what they drill in Match mode and lets them deal fresh rounds without leaving the screen.

## Description
Add scope, filter, and pair-count controls to `crates/studyfrog-ui/src/view/match_grid.rs` (splitting helper modules as needed to stay $\le 250$ LOC), wire `New round` alongside `Restart`, and correct the `Exit` tooltip.

## Acceptance Criteria
- [x] Match HUD/setup bar lets the user select Entire vault or a specific stack, filter (All, Due first, Least familiar), and pair count (4, 6, 8 pairs)
- [x] Restart reshuffles the current round while New round deals a fresh subset of pairs from the deck
- [x] The Exit button tooltip accurately reads `"Leaves Match mode."` instead of `"Quits the app."`

## Design Notes
- Use `palette_of(ui)` tokens and accessible `WidgetInfo` labels on all controls.

## Dependencies
- `STORY-0245`, `TASK-0301`

## Open Questions
None.
