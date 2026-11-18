---
id: STORY-0173
type: story
title: "Memory Session Engine, Card Deck Pairing, and Flip-State Transitions"
status: backlog
epic: EPIC-059
parents: ['EPIC-059']
estimate: 3
labels: [m14, rehearse]
acceptance_criteria:
  - 'Pure domain state machine in crates/studyfrog-rehearsal modeling MemoryTile, TileFlipState, and MemorySession'
  - 'Card deck pair generation extracting front and back into distinct tiles with unique pair IDs and deterministic shuffling'
  - 'Two-turn flip logic resolving match vs mismatch with configurable display delay and move counters'
---

## Context
Memory rehearsal requires a deterministic, pure state machine that converts flashcards into pairs of face-down tiles, tracks user flips, detects matches, and enforces delay timers on mismatches.

## Description
Develop the core domain model and state machine for Memory Mode:
- Define `TileRole` (`Front`, `Back`), `FlipState` (`FaceDown`, `FaceUp`, `MismatchWait`, `Matched`), and `MemoryTile`.
- Implement `memory_session_new` taking flashcard pairs, deck slice sizing (e.g., 6, 8, or 12 pairs), random seed, and countdown/stopwatch configuration.
- Implement `flip_tile` state transitions:
  - Turn 1: Reveal first tile.
  - Turn 2: Reveal second tile, evaluate pair identity, increment move count, update streak, and trigger either `Matched` or `MismatchWait`.
- Implement `tick_memory_timer` to advance elapsed time, handle mismatch reveal expirations (flipping back to `FaceDown`), and detect completion or timeout.

## Acceptance Criteria
- [ ] Pure domain state machine in crates/studyfrog-rehearsal modeling MemoryTile, TileFlipState, and MemorySession
- [ ] Card deck pair generation extracting front and back into distinct tiles with unique pair IDs and deterministic shuffling
- [ ] Two-turn flip logic resolving match vs mismatch with configurable display delay and move counters

## Design Notes
Keep all operations free functions with zero inherent `impl` blocks (Invariant P1). Cover pairing, shuffling, flip sequence, mismatch timers, and completion edge cases with comprehensive unit tests.

## Dependencies
EPIC-059

## Open Questions
None.
