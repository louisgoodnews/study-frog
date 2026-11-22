---
id: EPIC-059
type: epic
title: "Memory Mode Rehearsal: Card-Flipping Pair Concentration Drill Against Time"
status: review
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
- [ ] Interactive grid UI rendering face-down cards with flip transitions, match confirmations, and mismatch reset delays
- [x] Time-attack countdown and stopwatch modes challenging users to reunite all pairs against the clock
- [ ] Rehearsal entry point from stacks and sessions with score tracking, streaks, and personal best records

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

## Open Questions
None.
