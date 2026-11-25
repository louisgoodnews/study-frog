---
id: EPIC-070
type: epic
title: "UI redesign in the style of the owner's reference dashboards (gradients, card grids, stat rings, light and dark)"
status: in_progress
parents: []
labels: [m14, ui, docs]
acceptance_criteria:
  - "The owner's reference designs (four dashboards, 2026-11-25: a light purple NFT marketplace, a light purple job board, a dark music player and a dark analytics dashboard) are the visual target for layout, card style, sidebar, stat rings and the headline cards"
  - "Both the light and the dark appearance keep working, with the existing appearance setting (EPIC-027)"
  - "Every colour comes from the palette tokens (`palette_of`), never a raw literal, and every text and background pair meets the contrast criteria of EPIC-036 (APCA)"
  - "Gradients are decorative: a gradient never sits behind body text unless the text passes the contrast criteria on every point of it"
  - "The redesign is for desktop windows; no phone layout is built"
---

## Context
The owner wants the application to look like the reference dashboards: a left navigation rail, a hero card with an
image or gradient, lists of cards with avatars and ratings, a ring for statistics, and a highlighted card column on the
right. The current design is the calm UI (`docs/specs/ui-calm.md`, `docs/specs/ui-colour-and-type.md`), accepted by the
owner for 0.3.1 and later. This epic changes the look, not the behaviour of the screens.

## Description
Define the new visual system (card, sidebar, hero, ring, badge, avatar and gradient tokens) in the theme module, then
apply it screen by screen. Each screen keeps its function, its keyboard use and its control protocol commands.

## Acceptance Criteria
- [ ] Reference designs are the visual target, as described above
- [ ] Light and dark appearances both work through the existing setting
- [ ] Colour from palette tokens only; contrast per EPIC-036
- [ ] Gradients are decorative; no body text on a gradient that fails the contrast criteria
- [ ] Desktop only: no phone layout is built

## Design Notes
- The references are dashboards; StudyFrog screens are library, stack pages, Home, rehearsal and reports. The mapping of
  references to screens is an open question.
- Images in the references are other people's artwork; they are not copied into the app. Only the layout and the style
  are taken.

## Dependencies
- EPIC-027 (appearance), EPIC-036 (contrast and colour), `docs/specs/ui-calm.md` (current direction, to be revised by
  the owner's decision).

## Decisions (owner, 2026-11-25)
- Scope: Home and the stack page first; library, rehearsal and reports follow in later stories.
- Themes: light and dark, through the existing appearance setting.
- Direction: extend `docs/specs/ui-calm.md` (keep its spacing, reading width and fewer-items rules) and add the new card, gradient and ring styles on top.

## Open Questions
None. The scope and direction questions above are decided.
