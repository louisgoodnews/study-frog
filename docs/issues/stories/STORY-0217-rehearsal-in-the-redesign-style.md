---
id: STORY-0217
type: story
title: "Rehearsal in the redesign style: setup, card view and result"
status: review
epic: EPIC-070
parents: ['EPIC-070']
estimate: 5
labels: [m14, ui]
acceptance_criteria:
  - "The rehearsal setup, the card view and the result use the redesign tokens; the card stays the largest element on screen"
  - "Answering, rating, keyboard shortcuts and the control protocol are unchanged"
  - "Light and dark both pass the screenshot check at 1280 by 800; the answer text meets the contrast criteria of EPIC-036"
---

## Context
Rehearsal is the study moment, so the card view must keep its focus while the style changes.

## Description
Apply the tokens of STORY-0213 to the rehearsal screens. Presentation only.

## Acceptance Criteria
- [ ] Setup, card view and result in the redesign style
- [x] Answering, rating and shortcuts unchanged
- [ ] Screenshot check in both themes; contrast on the answer text

## Design Notes
No gradient behind the card text.

## Dependencies
- STORY-0213.

## Open Questions
None.

## Verification note
- Changed: the rehearsal setup's title, step and hint sit on the gradient panel (`view/setup.rs`); the result's header
  (Run finished / stopped, with Back to setup) sits on the gradient panel (`view/summary.rs`); the kit card's corner
  radius now reads the redesign token `CARD_RADIUS` (same value, 12). The card view itself stays on a plain surface
  with no gradient.
- Unchanged: answering, rating, shortcuts, messages and control commands.
- Checked: `CARBON_GATE` gate `scripts/check.sh` passed (EXIT 0, with CARGO_INCREMENTAL=0). The existing
  `tests/appearance_apca.rs` already checks text and weak text on every item-tinted card fill in all 24 palettes
  against the APCA targets, so no new test was added. Screenshots at 1280 by 800, dark and light: setup with the new
  gradient header, legible in both themes.
- Not checked: the card view and the result with a real item. The scratch vault has no items, so no populated
  rehearsal was captured. The exam result (`view/exam_result.rs`) was not restyled.

## Review note (2026-11-25)
Moved from done to review by the owner's rule that a done item has no unchecked criteria. Still open: the card view and the result with a real item (the scratch vault has no items, so no populated rehearsal was checked); the card view itself was not restyled (it already uses the same tokens, so only its look with an item is unverified); the answer-text contrast on the card has no new test (existing tests cover the item-tinted card fills).
