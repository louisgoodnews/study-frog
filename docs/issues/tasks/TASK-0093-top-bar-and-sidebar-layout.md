---
id: TASK-0093
type: task
title: Top bar, sidebar and menus
status: done
epic: EPIC-020
parents: [STORY-0036]
estimate: 5
labels: [m8, ui]
acceptance_criteria:
  - "The bar has three sections that keep their place: brand and switchers on the left, search and Create in the middle, icon buttons on the right; without a vault only the brand and help show"
  - "Sidebar lists the screens and folds away; the vault switcher lists recent vaults; Create goes where the new thing is made; Help lists the keys; Profile shows the version and closes the vault"
  - "Every icon is drawn with shapes (the bundled fonts have no emoji and egui panics on a missing glyph) and has an accessible name"
---

## Context


## Description
Top bar, sidebar and menus.

## Acceptance Criteria
- [x] The bar has three sections that keep their place: brand and switchers on the left, search and Create in the middle, icon buttons on the right; without a vault only the brand and help show
- [x] Sidebar lists the screens and folds away; the vault switcher lists recent vaults; Create goes where the new thing is made; Help lists the keys; Profile shows the version and closes the vault
- [x] Every icon is drawn with shapes (the bundled fonts have no emoji and egui panics on a missing glyph) and has an accessible name

## Design Notes
`ui::chrome` (pure state and update), `view::{navbar, icons, popup, menus, sidebar}`. A menu closes on Escape or a click outside. Checked headless and by looking at the real window.

## Dependencies
None.

## Open Questions
None.
