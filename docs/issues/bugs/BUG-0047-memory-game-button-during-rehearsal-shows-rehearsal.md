---
id: BUG-0047
type: bug
title: "Memory game on a stack page shows the open rehearsal instead of starting a memory board"
status: done
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

**Fix (2026-11-22).** The rehearse screen shows a memory game over an open run: the route in `view/mod.rs` uses `setup/memory.rs` `run_on_screen`, which gives the board the page while a game is on screen, and the run waits underneath without being ended. Closing the game brings the run back. The board says that a run is open (`view/setup.rs`), so the button neither ends the run nor shows it in place of the game. A new run clears a finished game left on screen (`run_update::begin_run`). Test: `studyfrog-ui/tests/memory_entry.rs` `a_memory_game_takes_the_page_over_an_open_run` and `closing_the_memory_game_brings_the_open_run_back`. Running app: before, the Memory game button showed the run ("front", 1 / 3) (`/tmp/claude-0/memory/before-047-memory-during-run.png`); after, it shows the board with the line "A rehearsal run is open. It resumes when you close this game." (`/tmp/claude-0/memory/after-047-memory-during-run.png`), and Close game returns the run (`/tmp/claude-0/memory/after-047-closed-run-resumes.png`).

## Description
A Memory game start should either take over the screen with a new board or refuse and say why. It should not show a run the user did not ask for. Check the path that opens the Rehearse screen from the stack page's Memory game button: it keeps the existing run's screen and does not reset to the game setup.

## Acceptance Criteria
- [x] Memory game on a stack page starts a memory board for that stack when an ordinary rehearsal run is open
- [x] With a run open, the button asks before ending that run or says a run is open, and never shows the run in place of the game

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/view/stack_view.rs` (the "Memory game" button), `crates/studyfrog-ui/src/view/setup.rs`, `crates/studyfrog-ui/src/setup/memory.rs`.
- Guardrails: P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- STORY-0175
- BUG-0024

## Verification (2026-11-22)

Review (2026-11-22): NOT FIXED. With an ordinary rehearsal open (4/6, front4), Memory game on the Memo stack page shows the rehearsal instead of a board (`/tmp/claude-0/review-c/shots/17-memory-during-run.png`). Both criteria fail. Stays backlog.

## Open Questions
- None.
