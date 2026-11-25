---
id: STORY-0226
type: story
title: "Violet to pink palette tokens for light and dark, all six accents kept"
status: ready
epic: EPIC-073
parents: ['EPIC-073']
estimate: 5
labels: [m14, ui]
acceptance_criteria:
  - "The palette values for light and dark, default violet accent and pink secondary, are defined in theme_tokens.rs"
  - "The six accent choices remain, each with the same lightness and chroma rule as before"
  - "The gradient pair used by theme/style.rs is violet to pink"
  - "The APCA tests of tests/appearance_apca.rs are the acceptance check, to be run in the test phase"
---

## Context
EPIC-073 (whole-app redesign with a new theme, violet to pink as the identity, all six accents kept; decided 2026-11-25).

## Description
Replace the palette values in crates/studyfrog-ui/src/theme_tokens.rs with the violet-to-pink identity: violet as the default accent, pink as the secondary, neutral surfaces tinted towards lavender in light and deep blue-grey in dark. Keep the OKLCH structure, the six accents and the high-contrast variant. Values are unverified until the test phase.

## Acceptance Criteria
- None; EPIC-073 direction decided.

## Design Notes
Colours come from palette_of / style_of, never literals. Contrast targets are those of tests/appearance_apca.rs (APCA, EPIC-036).

## Dependencies


## Open Questions
None.
