---
id: STORY-0147
type: story
title: Research and prototype floating burger navigation menu
status: backlog
epic: EPIC-047
parents: [EPIC-047]
estimate: 3
labels: [m12, ui, ux, research]
acceptance_criteria:
  - "UX research document evaluates ergonomic trade-offs between static top bar / sidebar navigation versus a floating corner burger menu"
  - "Prototype demonstrates collapsible floating navigation button providing quick access to all application screens"
  - "The design ensures non-interference with card content, reading measure widths, and touch/click affordances"
---

## Context
As StudyFrog adds features (timetable, prompt library, python console, active recall modes), the top navbar and sidebar risk visual crowding. The user requested researching moving navigation items into a floating burger button to maximize distraction-free reading area while retaining rapid screen switching.

## Description
Conduct UX research and develop a prototype:
1. Author research report `docs/research/floating-burger-navigation.md` analyzing spatial footprint, Fitts's law target acquisition, Zen mode synergy, and keyboard accessibility.
2. Prototype a floating action button / burger trigger that opens a compact radial or popover navigation sheet with instant shortcuts.
3. Align with accessibility guidelines and screen reader navigation.

## Acceptance Criteria
- [ ] UX research document evaluates ergonomic trade-offs between static top bar / sidebar navigation versus a floating corner burger menu
- [ ] Prototype demonstrates collapsible floating navigation button providing quick access to all application screens
- [ ] The design ensures non-interference with card content, reading measure widths, and touch/click affordances

## Design Notes
- Touchpoints: `docs/research/`, `crates/studyfrog-ui/src/view/`.
- Guardrails: P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- EPIC-047 (Brand mascot icon and consolidated navigation UX).

## Open Questions
- None.
