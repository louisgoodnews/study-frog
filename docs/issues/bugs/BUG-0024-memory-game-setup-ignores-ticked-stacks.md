---
id: BUG-0024
type: bug
title: "The Memory game button in the rehearse setup ignores the stacks ticked in the What step"
status: backlog
estimate: 2
labels: [m14, ui, rehearse]
epic: EPIC-059
parents: ['EPIC-059']
acceptance_criteria:
  - "With a stack ticked in the rehearse setup What step, Memory game plays that stack even when another stack is open in the library"
  - "The Memory game button shows an error only when the ticked stack has fewer than two usable cards, and names that stack"
---

## Context
Found while auditing STORY-0175 / EPIC-059 in the running app (2026-11-22, display :91). The Memory game button in the rehearse setup plays the stack open in the library, not the stacks ticked under Stacks. Reproduced: Bio ticked, Geo (one card) open in Items, Memory game shows "This stack needs at least two cards to play." Screenshots: `/tmp/claude-0/audit/g1/04-after-close.png` (Bio ticked, 60 s selected), `/tmp/claude-0/audit/g1/03-memory-setup.png` (the error). Code: `crates/studyfrog-ui/src/setup/memory.rs` `fn start` reads `state.browser.selection`, not the rehearse state's ticked stacks.

## Description
Make the rehearse setup's Memory game button use the ticked stack, or hide the stack checkboxes for it and say which stack it uses. Keep the stack-page button as it is.

Reproduction: 1) Open Rehearse. 2) Tick Bio under Stacks. 3) In Items, open Geo (one card). 4) Back in Rehearse, click Memory game. Actual: the error above. Expected: the game plays Bio.

## Acceptance Criteria
- [ ] With a stack ticked in the rehearse setup What step, Memory game plays that stack even when another stack is open in the library
- [ ] The Memory game button shows an error only when the ticked stack has fewer than two usable cards, and names that stack

## Design Notes
- None.

## Dependencies
- STORY-0175 / EPIC-059

## Open Questions
- None.
