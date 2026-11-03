---
id: TASK-0225
type: task
title: Accent colour, high contrast and reduced motion
status: backlog
epic: EPIC-027
parents: [STORY-0081]
estimate: 5
labels: [m11, ui]
acceptance_criteria:
  - "An accent colour can be chosen from a palette that works in both themes"
  - "A high-contrast variant of each theme has text and borders at a ratio of 7 to 1"
  - "Reduced motion turns off animations (the scroll and the menus' fades) and is on when the system asks for it"
  - "Meaning is never carried by colour alone, as today"
---

## Context
The owner asked on 2026-11-03 for a dark and a light mode and for UI settings in the application, and for an epic for them.

## Description
Choices for people who want another colour or need more contrast.

## Acceptance Criteria
- [ ] An accent colour can be chosen from a palette that works in both themes
- [ ] A high-contrast variant of each theme has text and borders at a ratio of 7 to 1
- [ ] Reduced motion turns off animations (the scroll and the menus' fades) and is on when the system asks for it
- [ ] Meaning is never carried by colour alone, as today

## Design Notes
Palette entries as data with names; the contrast test of TASK-0226 covers the new variants.

## Dependencies
TASK-0222 and TASK-0226.

## Open Questions
None.
