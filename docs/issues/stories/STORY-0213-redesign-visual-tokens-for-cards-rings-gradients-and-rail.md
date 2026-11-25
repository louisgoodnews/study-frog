---
id: STORY-0213
type: story
title: "Redesign visual tokens: cards, stat rings, gradient headline cards, avatars and the navigation rail"
status: done
epic: EPIC-070
parents: ['EPIC-070']
estimate: 5
labels: [m14, ui]
acceptance_criteria:
  - "The theme defines tokens for the card (radius, elevation, border), the navigation rail, the stat ring, the gradient headline card, the avatar and the badge, all from the palette, for both light and dark"
  - "Every gradient token has a plain colour fallback; the contrast check of EPIC-036 runs on every text pair a token is used with, and the build fails if one fails"
  - "No new raw colour literals outside the theme module"
  - "A 16 px gutter token is defined and tested"
---

## Context
EPIC-070 asks for the reference style. The style needs a token set before any screen uses it, so every screen uses the
same card, ring and gradient.

## Description
Add the tokens to `theme.rs`, the spacing ones to `theme_space.rs`, and the contrast checks to `contrast.rs`. Keep the
calm UI spacing rules (`docs/specs/ui-calm.md`).

## Acceptance Criteria
- [x] Tokens for card, rail, ring, gradient card, avatar and badge in light and dark
- [x] Gradient fallbacks, and the contrast check covers every token that carries text
- [x] No raw colour literals outside `theme.rs`
- [x] A 16 px gutter token is defined and tested

## Design Notes
Build the ring with the existing painter, not a new dependency (P10). Take the layout from the reference dashboards; take
no artwork.

## Dependencies
- EPIC-070, EPIC-036 (contrast), EPIC-027 (appearance).

## Open Questions
None.

## Verification note
Done 2026-11-25. The tokens are in `crates/studyfrog-ui/src/theme/style.rs`, derived from the palette with `blend` in `theme.rs`. `crates/studyfrog-ui/tests/style_tokens.rs` checks the text on the gradient at both ends, the badge and rail text, the card outline and the ring against the APCA targets of `appearance_apca`, in all 24 palettes. `no_hardcoded_colours` passes. `scripts/check.sh` passed on the tree before this commit.
