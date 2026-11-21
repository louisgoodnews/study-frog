---
id: STORY-0174
type: story
title: "Memory Mode Interactive Card Grid UI, Card Back Styling, and Audio-Visual Feedback"
status: review
epic: EPIC-059
parents: ['EPIC-059']
estimate: 3
labels: [m14, ui, rehearse]
acceptance_criteria:
  - 'Responsive card grid view dynamically sizing tiles based on window dimensions and pair count'
  - 'Distinguished card-back visual styling featuring StudyFrog frog branding or subtle geometric patterns'
  - 'Clear visual feedback states for hovered cards, face-up text reading, successful match glows, and mismatch shakes/warnings'
---

## Context
A physical memory game depends heavily on visual tactile clarity: cards must look distinct when face-down, flip cleanly to reveal legible text, and visually signal success or error when two cards are flipped.

## Description
Build the interactive user interface for Memory Mode in `crates/studyfrog-ui`:
- Render the tile grid with responsive cell dimensions adapting to 3x4, 4x4, or 4x5 layouts.
- Design face-down card backs using `palette_of(ui)` theme tokens, featuring a branded frog motif or patterned border.
- Render flipped cards with clear typography, wrapping, and subtle elevation/shadows.
- Provide clear color-coded feedback:
  - Accent glow and persistent state for matched pairs.
  - Danger/warning stroke and shake/pause state for mismatches prior to flipping back down.
- Maintain HUD showing elapsed time, moves made, pairs remaining, and current streak.

## Acceptance Criteria
- [x] Responsive card grid view dynamically sizing tiles based on window dimensions and pair count
- [x] Distinguished card-back visual styling featuring StudyFrog frog branding or subtle geometric patterns
- [x] Clear visual feedback states for hovered cards, face-up text reading, successful match glows, and mismatch shakes/warnings

## Implementation (2026-11-21)
The game is on the rehearse screen: "Memory game" in the What step starts it from the stack open in the browser (the inbox works too). The stack's first cards (up to 40) are read by `exec_memory.rs`; the game uses up to eight pairs of the usable ones. `setup/memory.rs` holds the state and its pure update; the rules come from the engine in `studyfrog-rehearsal`.

`setup/memory_layout.rs` picks the column count (2 to 8) that gives the largest cards at the card aspect, so 12, 16 and 20 cards fill the window with 3, 4 or 5 columns. The board (`view/memory_board.rs`) paints face-down cards with a dot lattice in palette tokens, face-up text wrapped inside the card, a success glow on matches, and a danger outline that shakes on a mismatch. The HUD shows time, moves, pairs left and the streak; the end line offers "Play again". A clock message is sent each frame while a game runs, so the mismatch delay runs on the wall clock.

Tests: `studyfrog-ui/tests/memory_mode.rs` (grid fit and aspect, start and load paths, the eight-pair limit, the mismatch delay, close). The on-screen look is not yet checked in a screenshot: the screenshot tool cannot open a running game.

## Design Notes
Use `egui` drawing primitives, strictly adhering to `palette_of(ui)` without hardcoded RGB values. Ensure file line counts remain $\le 250$ LOC per file (Invariant P6).

## Dependencies
EPIC-059, STORY-0173

## Open Questions
None.
