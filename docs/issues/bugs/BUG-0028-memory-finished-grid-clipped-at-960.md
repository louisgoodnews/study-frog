---
id: BUG-0028
type: bug
title: "At 960 by 640 the Memory board under the win message runs past the bottom of the window"
status: backlog
estimate: 2
labels: [m14, ui, rehearse]
epic: EPIC-059
parents: ['EPIC-059']
acceptance_criteria:
  - "At 960 by 640, after a win or a timeout, every card of the grid can be seen (the grid fits below the message, or it scrolls)"
---

## Context
Found while auditing EPIC-059 (2026-11-22, display :91). At 960 by 640, after a win, the end line and the Play again button sit above the grid, which is laid out with seven columns; its last row (Back 5, Front 2) is cut off by the status strip at the bottom. Screenshot: `/tmp/claude-0/audit/g1/18-board-960.png`. A new game at the same size fits: `/tmp/claude-0/audit/g1/19-newgame-960.png`.

## Description
Give the grid the height left below the message, or let the board scroll. Reproduction: at 960 by 640 finish a Memory game, or let one time out.

## Acceptance Criteria
- [ ] At 960 by 640, after a win or a timeout, every card of the grid can be seen (the grid fits below the message, or it scrolls)

## Design Notes
- None.

## Dependencies
- STORY-0175 / EPIC-059

## Open Questions
- None.
