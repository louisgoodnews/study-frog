---
id: STORY-0170
type: story
title: "Menu De-Cluttering, Visual Spacing, and Progressive Disclosure"
status: backlog
epic: EPIC-057
parents: ['EPIC-057']
estimate: 5
labels: [m14, ui, ux]
acceptance_criteria:
  - 'Header bars reduce visible primary buttons to at most 3 core actions, collapsing secondary tools into an overflow menu'
  - 'Increased margin and row spacing across lists, forms, and dialogs preventing cramped visual density'
  - 'Clear typographic hierarchy establishing an airy, modern, distraction-free aesthetic'
---

## Context
Menus in StudyFrog are overpopulated, creating visual fatigue and making the program feel like an intimidating technical utility rather than an enjoyable learning environment.

## Description
De-clutter screens and menus across the entire application:
- Identify crowded headers (e.g. stack view, item editor, PDF viewer) and consolidate low-frequency utility buttons into an overflow dropdown menu (`...`).
- Increase comfortable whitespace around list items, action buttons, and modal forms.
- Ensure the primary action on every screen stands out cleanly without competing visual noise.

## Acceptance Criteria
- [ ] Header bars reduce visible primary buttons to at most 3 core actions, collapsing secondary tools into an overflow menu
- [ ] Increased margin and row spacing across lists, forms, and dialogs preventing cramped visual density
- [ ] Clear typographic hierarchy establishing an airy, modern, distraction-free aesthetic

## Design Notes
Preserve all keyboard shortcuts even when secondary buttons are folded into overflow menus. Maintain full screen reader accessibility.

## Dependencies
EPIC-057

## Open Questions
None.
