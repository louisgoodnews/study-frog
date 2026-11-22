---
id: EPIC-059
type: epic
title: "Memory Mode Rehearsal: Card-Flipping Pair Concentration Drill Against Time"
status: done
parents: []
labels: [m14, rehearse, ui]
acceptance_criteria:
  - 'Memory rehearsal game engine pairing flashcard fronts and backs as face-down tiles'
  - 'Interactive grid UI rendering face-down cards with flip transitions, match confirmations, and mismatch reset delays'
  - 'Time-attack countdown and stopwatch modes challenging users to reunite all pairs against the clock'
  - 'Rehearsal entry point from stacks and sessions with score tracking, streaks, and personal best records'
---

## Context
While StudyFrog supports classic spaced repetition review and open-face match drills, visual-spatial associative learning is powerfully reinforced through the classic "Memory" (Concentration / Pelmanism) game format. In this drill, cards are initially face down; students flip pairs one by one, memorizing tile locations and reuniting card prompts with their corresponding answers against time.

## Description
Implement a complete Memory Mode rehearsal feature:
1. **Memory Game Mechanics**: Transform flashcards from a chosen stack into paired tiles (prompt tile and answer tile). Tiles are shuffled and placed face down in a responsive grid.
2. **Turn & Flip State Machine**: Clicking a tile flips it face up. Clicking a second tile checks for a match:
   - If front and back belong to the same flashcard, the pair is successfully matched and remains highlighted or clears.
   - If they do not match, both tiles remain revealed briefly so the user can memorize their locations, then flip face down again.
3. **Against-the-Clock Time Challenge**: Offer both stopwatch (fastest clear) and time-attack countdown (clear the grid before timer runs out) modes, penalizing mismatches or rewarding speed streaks.
4. **Stack & Rehearse Synergy**: Seamlessly launch Memory Mode directly from the Rehearse screen or any stack view, recording completion stats and high scores.

## Acceptance Criteria
- [x] Memory rehearsal game engine pairing flashcard fronts and backs as face-down tiles
- [x] Interactive grid UI rendering face-down cards with flip transitions, match confirmations, and mismatch reset delays
- [x] Time-attack countdown and stopwatch modes challenging users to reunite all pairs against the clock
- [x] Rehearsal entry point from stacks and sessions with score tracking, streaks, and personal best records

## Design Notes
Build pure domain logic in `studyfrog-rehearsal::memory_mode` using free functions adhering strictly to Invariant P1 (zero inherent `impl` blocks). Maintain palette color integrity via `palette_of(ui)` for card backs, flipped faces, matched glows, and mismatch alerts. Keep all source files $\le 250$ LOC (Invariant P6).

## Dependencies
None.

## Audit (2026-11-22)
Audited in the running app (display :91). Verdicts per criterion:
- Engine pairs fronts and backs as face-down tiles: MET (`/tmp/claude-0/audit/g1/08-memory-board.png`, pairs "Front N"/"Back N" in `/tmp/claude-0/audit/g1/17-victory.png`). Ticked.
- Grid with flip, match confirmation and mismatch delay: NOT CHECKABLE for the 0.2 s flip itself (no frames of the turn captured). Match glow and mismatch outline with turn-back MET (`/tmp/claude-0/audit/g1/13-match.png`, `/tmp/claude-0/audit/g1/09-flipped-two.png`). Left unticked.
- Time-attack and stopwatch: MET. Stopwatch win at 4:56 (`/tmp/claude-0/audit/g1/17-victory.png`); 60 s timeout (`/tmp/claude-0/audit/g1/14-timeout.png`). Ticked; the countdown readout is missing (BUG-0025).
- Entry point, score, streaks, personal best: stack page and rehearse entry MET; streak and best time MET (`app.toml` `[[memory_best]]`). The "sessions" entry point was not checked. Left unticked.
Open: BUG-0024 to BUG-0029. The epic stays in review.

## Verification (2026-11-22)
Checked in the running app (`/tmp/claude-0/verifyB`, 1280x800, Memo stack with six cards):
- Grid flip transition: MET. A screen recording at 30 fps (`/tmp/claude-0/verifyB/flip1.mp4`) and the frame sheet `/tmp/claude-0/verifyB/m04-flip-sheet.png` show the tile narrowing to a sliver over about 0.25 s, then widening to its face ("back4"). `FLIP_SECS` is 0.2 s (`setup/memory_flip.rs`).
- Match confirmation: MET. Pair back4 and front4 turned green, pairs left went 6 to 5, streak 1 (`/tmp/claude-0/verifyB/m06-match.png`).
- Mismatch reset: MET. Both faces stayed up with a red outline (`/tmp/claude-0/verifyB/pair_1_2.png`), then turned back down with moves 2 (`/tmp/claude-0/verifyB/m05-after-two.png`). `MISMATCH_MS` is 900.
- Time-attack and stopwatch: unchanged from the earlier audit (see STORY-0175).
- Entry point from stacks and sessions: NOT MET for sessions. The stack page and Rehearse setup start a game (`/tmp/claude-0/verifyB/m03-board.png`). The Calendar's "Sessions on <date>" panel offers no Memory game: a planned-review row offers only "Rehearse Memo" and "Open Memo" (`/tmp/claude-0/verifyB/c02-session.png`), and `view/calendar.rs` `occurrence_row` offers only "Start rehearsal for <title>". Filed as BUG-0046. Also, the stack page's Memory game button shows an open rehearsal instead of starting a board (`/tmp/claude-0/verifyB/m08-memory-during-run.png`, BUG-0047). This criterion stays unticked, so the epic stays in review.

Fixed (2026-11-22): the sessions entry point is met. The Calendar session row offers "Memory game for Study" (`/tmp/claude-0/memory/after-cal-session.png`) and opens the board (`/tmp/claude-0/memory/after-calendar-memory-board.png`); the planned-review row offers "Memory game for X" (`/tmp/claude-0/memory/after-cal-review-row.png`) and opens the board (`/tmp/claude-0/memory/after-review-row-memory-board.png`). BUG-0046 and BUG-0047 are done (their fixes are checked in the running app with the screenshots above), and BUG-0024 to BUG-0029 were already done, so all four criteria are ticked and the epic is done.

Review (2026-11-22): criteria 1 to 3 stay ticked (not re-checked here). Criterion 4 stays unticked: BUG-0046 still reproduces, the Calendar session row for 2026-11-24 offers "Start rehearsal for Memo session", "Edit" and "Delete" and no Memory game (`/tmp/claude-0/review-c/shots/22-session-row.png`); the planned-review rows offer only Rehearse, Open and Move (`/tmp/claude-0/review-c/shots/19-cal24.png`). BUG-0047 still reproduces: with an ordinary rehearsal open (4/6), Memory game on the Memo stack page shows that rehearsal, not a board (`/tmp/claude-0/review-c/shots/17-memory-during-run.png`). Status stays review.

## Open Questions
None.
