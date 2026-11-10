---
id: STORY-0146
type: story
title: Stylized Breviceps macrops brand mascot icon
status: done
epic: EPIC-047
parents: [EPIC-047]
estimate: 2
labels: [m12, ui, brand]
acceptance_criteria:
  - "The navbar brand icon renders a stylized Breviceps macrops (desert rain frog) with its signature round spherical silhouette and expressive large eyes"
  - "The icon renders cleanly at 24x24 and scales sharply to larger resolutions using vector primitives"
  - "The icon matches the OKLCH theme palette in both light and dark modes"
---

## Context
The current app logo in the top bar is a basic set of green circles. The user requested replacing this with StudyFrog's true mascot: a stylized *Breviceps macrops* (the desert rain frog), known for its adorable round shape, serious studious demeanor, and endearing forward-facing eyes.

## Description
Redesign the frog icon in `crates/studyfrog-ui/src/view/navbar.rs`:
1. Vectorize a stylized *Breviceps macrops* using epaint shapes:
   - Globular stout body ellipse with sandy/moss-green gradient or theme accent tint.
   - Large, prominent, expressive round eyes with catchlight highlights.
   - Distinctive studious/determined mouth line.
2. Ensure sharpness across 1x, 2x, and fractional zoom levels without raster blur.
3. Verify visual contrast against dark and light navbar surfaces.

## Acceptance Criteria
- [x] The navbar brand icon renders a stylized Breviceps macrops (desert rain frog) with its signature round spherical silhouette and expressive large eyes
- [x] The icon renders cleanly at 24x24 and scales sharply to larger resolutions using vector primitives
- [x] The icon matches the OKLCH theme palette in both light and dark modes

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/view/navbar.rs`.
- Guardrails: P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- EPIC-047 (Brand mascot icon and consolidated navigation UX).

## Open Questions
- None.
