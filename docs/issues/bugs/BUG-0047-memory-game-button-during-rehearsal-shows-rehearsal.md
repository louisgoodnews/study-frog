---
id: BUG-0047
type: bug
title: "Memory game on a stack page shows the open rehearsal instead of starting a memory board"
status: backlog
estimate: 2
labels: [m14, ui, rehearse]
epic: EPIC-059
parents: ['EPIC-059']
acceptance_criteria:
  - "Memory game on a stack page starts a memory board for that stack even when an ordinary rehearsal run is open"
  - "When a rehearsal run is open, the Memory game button either asks before ending that run or says that a run is open, and never shows the run in place of the game"
---

## Context
Found while verifying EPIC-059 (2026-11-22, `/tmp/claude-0/verifyB`). With a practice rehearsal of stack Alpha open (run paused at item 1 of 9, "Flip cards"), I opened stack Memo and clicked "Memory game". The screen showed the open rehearsal (`/tmp/claude-0/verifyB/m08-memory-during-run.png`, "1 / 9", "afront1"), not a memory board for Memo. With no run open, the same click starts the board (`/tmp/claude-0/verifyB/m03-board.png`).

## Description
A Memory game start should either take over the screen with a new board or refuse and say why. It should not show a run the user did not ask for. Check the path that opens the Rehearse screen from the stack page's Memory game button: it keeps the existing run's screen and does not reset to the game setup.

## Acceptance Criteria
- [ ] Memory game on a stack page starts a memory board for that stack when an ordinary rehearsal run is open
- [ ] With a run open, the button asks before ending that run or says a run is open, and never shows the run in place of the game

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/view/stack_view.rs` (the "Memory game" button), `crates/studyfrog-ui/src/view/setup.rs`, `crates/studyfrog-ui/src/setup/memory.rs`.
- Guardrails: P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- STORY-0175
- BUG-0024

## Open Questions
- None.
