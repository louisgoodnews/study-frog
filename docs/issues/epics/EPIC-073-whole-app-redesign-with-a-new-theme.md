---
id: EPIC-073
type: epic
title: "Whole-app redesign with a new theme: a new palette, applied to every screen"
status: backlog
parents: []
labels: [m14, ui, docs]
acceptance_criteria:
  - "A new palette is defined in the theme tokens for the light and dark themes, with the accent and status colours, and every screen takes its colours from it"
  - "The contrast criteria of EPIC-036 (APCA) are met by the new palette in every theme, contrast and accent combination, checked by the existing appearance tests"
  - "Every screen of the application uses the redesign's cards, rail and gradient panel, including those not yet redesigned"
  - "The owner has chosen the palette (the colours) before the palette story is started"
---

## Context
The owner asked for a whole-app redesign with a new theme (2026-11-25). EPIC-036 calibrated the current palette in
OKLCH and verified it against APCA. A new theme replaces those values, so the contrast tests of EPIC-036 must hold for
the new ones.

## Description
Two steps: define the new palette in the theme tokens (after the owner chooses the colours), then apply the redesign to
every remaining screen (Settings, Help, Calendar, Board, Diary, Achievements, Graph, Assistant, Classroom, and the
exam screens).

## Acceptance Criteria
- [ ] New palette defined in the theme tokens for both themes
- [ ] APCA contrast tests pass for the new palette
- [ ] Every screen uses the redesign components
- [ ] The owner has chosen the palette colours

## Design Notes
- Colour values are not chosen here. Open question below.
- The redesign components are in `view/gradient_panel.rs`, `view/home_hero.rs` (card) and `theme/style.rs`.

## Dependencies
- EPIC-036 (contrast), EPIC-070 (redesign components), EPIC-027 (appearance).

## Open Questions
- Which palette: the owner's colours or a proposal from the agents for the owner to accept or change?
