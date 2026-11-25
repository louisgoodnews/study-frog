---
id: STORY-0227
type: story
title: "Settings and Help in the redesign style"
status: ready
epic: EPIC-073
parents: ['EPIC-073']
estimate: 3
labels: [m14, ui]
acceptance_criteria:
  - "Settings and Help use the redesign cards, the rail-style navigation and the gradient headline"
  - "Every setting, link and tab keeps its behaviour and control-protocol answer"
  - "Tooltips on every text button"
---

## Context
EPIC-073 (whole-app redesign with a new theme, violet to pink as the identity, all six accents kept; decided 2026-11-25).

## Description
Apply the redesign components (view/gradient_panel.rs, view/home_hero.rs card, theme/style.rs) to the Settings and Help screens. Presentation only.

## Acceptance Criteria
- STORY-0226.

## Design Notes
Colours come from palette_of / style_of, never literals. Contrast targets are those of tests/appearance_apca.rs (APCA, EPIC-036).

## Dependencies


## Open Questions
None.
