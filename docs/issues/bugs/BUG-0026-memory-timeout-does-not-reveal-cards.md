---
id: BUG-0026
type: bug
title: "The Memory timeout message says the cards are shown, but the unmatched cards stay face down"
status: done
estimate: 1
labels: [m14, ui, rehearse]
epic: EPIC-059
parents: ['EPIC-059']
acceptance_criteria:
  - "After a time-attack timeout, every unmatched card is turned face up so the player can see where the pairs were"
  - "The timeout message matches what is on the board"
---

## Context
Found while auditing EPIC-059 (2026-11-22, display :91). The timeout line reads "Time is up. Here are the cards: try again and beat your time.", but the unmatched cards stay face down. Screenshot: `/tmp/claude-0/audit/g1/14-timeout.png` (only the matched pair is face up). EPIC-059's description asks for "revealed cards" on defeat.

**Fix (2026-11-22).** A timeout turns every unmatched card face up (`studyfrog-rehearsal/src/memory.rs` `time_out`), so the message matches the board. Tests: `studyfrog-rehearsal/tests/memory.rs` `a_timeout_turns_the_unmatched_cards_face_up`; `studyfrog-ui/tests/memory_mode.rs` `a_time_attack_timeout_reveals_the_unmatched_cards`. Running app: after the 60 s timeout all unmatched cards are face up (`/tmp/claude-0/fix1/0026-0028-timeout-after-960.png`).

## Description
Reveal the unmatched cards when the game times out, or change the message. Reproduction: start a 60 s time-attack game on a stack of 8 or more cards and do not finish it; after 60 s the unmatched cards are still face down.

## Acceptance Criteria
- [x] After a time-attack timeout, every unmatched card is turned face up so the player can see where the pairs were
- [x] The timeout message matches what is on the board

## Design Notes
- None.

## Dependencies
- STORY-0175 / EPIC-059

## Open Questions
- None.
