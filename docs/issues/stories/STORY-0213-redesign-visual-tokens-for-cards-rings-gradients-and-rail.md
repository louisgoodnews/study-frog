---
id: STORY-0213
type: story
title: "Redesign visual tokens: cards, stat rings, gradient headline cards, avatars and the navigation rail"
status: backlog
epic: EPIC-070
parents: ['EPIC-070']
estimate: 5
labels: [m14, ui]
acceptance_criteria:
  - "The theme defines tokens for the card (radius, elevation, border), the navigation rail, the stat ring, the gradient headline card, the avatar and the badge, all from the palette, for both light and dark"
  - "Every gradient token has a plain colour fallback; the contrast check of EPIC-036 runs on every text pair a token is used with, and the build fails if one fails"
  - "No new raw colour literals outside the theme module"
  - "The tokens work at phone width with a 16 px gutter"
---

## Context
EPIC-070 asks for the reference style. The style needs a token set before any screen uses it, so every screen uses the
same card, ring and gradient.

## Description
Add the tokens to `theme.rs`, the spacing ones to `theme_space.rs`, and the contrast checks to `contrast.rs`. Keep the
calm UI spacing rules (`docs/specs/ui-calm.md`).

## Acceptance Criteria
- [ ] Tokens for card, rail, ring, gradient card, avatar and badge in light and dark
- [ ] Gradient fallbacks, and the contrast check covers every token that carries text
- [ ] No raw colour literals outside `theme.rs`
- [ ] Phone width with a 16 px gutter

## Design Notes
Build the ring with the existing painter, not a new dependency (P10). Take the layout from the reference dashboards; take
no artwork.

## Dependencies
- EPIC-070, EPIC-036 (contrast), EPIC-027 (appearance).

## Open Questions
None.
