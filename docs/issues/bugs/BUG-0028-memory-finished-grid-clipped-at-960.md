---
id: BUG-0028
type: bug
title: "At 960 by 640 the Memory board under the win message runs past the bottom of the window"
status: done
estimate: 2
labels: [m14, ui, rehearse]
epic: EPIC-059
parents: ['EPIC-059']
acceptance_criteria:
  - "At 960 by 640, after a win or a timeout, every card of the grid can be seen (the grid fits below the message, or it scrolls)"
---

## Context
Found while auditing EPIC-059 (2026-11-22, display :91). At 960 by 640, after a win, the end line and the Play again button sit above the grid, which is laid out with seven columns; its last row (Back 5, Front 2) is cut off by the status strip at the bottom. Screenshot: `/tmp/claude-0/audit/g1/18-board-960.png`. A new game at the same size fits: `/tmp/claude-0/audit/g1/19-newgame-960.png`.

**Fix (2026-11-22).** The Memory board is no longer inside an unbounded scroll area: the header (HUD, message, Play again) is drawn first and the grid takes the height left below it (`view/setup.rs` memory branch, `view/memory/board.rs`). Running app at 960 by 640: after the 60 s timeout all 16 cards are visible above the status strip (`/tmp/claude-0/fix1/0026-0028-timeout-after-960.png`). Caveat: the pre-fix build also fitted in my timeout run (`/tmp/claude-0/fix1/0026-0028-timeout-before-960.png`), so the clipping in the audit screenshot depends on a layout I did not reproduce. I did not play a 16-tile win; a win shows the same header and grid as the timeout.

## Description
Give the grid the height left below the message, or let the board scroll. Reproduction: at 960 by 640 finish a Memory game, or let one time out.

## Acceptance Criteria
- [x] At 960 by 640, after a win or a timeout, every card of the grid can be seen (the grid fits below the message, or it scrolls)

## Design Notes
- None.

## Dependencies
- STORY-0175 / EPIC-059

## Open Questions
- None.
