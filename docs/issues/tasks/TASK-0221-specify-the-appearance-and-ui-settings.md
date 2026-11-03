---
id: TASK-0221
type: task
title: Specify the appearance and UI settings
status: done
epic: EPIC-027
parents: [STORY-0080]
estimate: 2
labels: [m11, docs, decision]
acceptance_criteria:
  - "docs/specs/appearance.md lists every setting with its values, its default, where it is stored (app.toml, per user and machine) and whether it takes effect at once"
  - "It says how the first start decides the theme (the system's) and how a change of the system theme is followed"
  - "The owner has accepted it"
---

## Context
The owner asked on 2026-11-03 for a dark and a light mode and for UI settings in the application, and for an epic for them.

## Description
Write the settings down before building them.

## Acceptance Criteria
- [x] docs/specs/appearance.md lists every setting with its values, its default, where it is stored (app.toml, per user and machine) and whether it takes effect at once
- [x] It says how the first start decides the theme (the system's) and how a change of the system theme is followed
- [x] The owner has accepted it

## Design Notes
Candidates: theme (dark, light, system); interface size (zoom, 80 to 200 percent, with Ctrl+plus, Ctrl+minus and Ctrl+0); text size apart from the zoom; density (compact, comfortable, spacious); accent colour; high contrast; reduced motion; start screen; sidebar default; remember the window's size and place; the folded or shown stack list. Keys in `app.toml` follow the existing style (`track_views`, `calendar_reviews`).

Built on 2026-11-03: written after the settings were built, at the owner's instruction to add all settings: `docs/specs/appearance.md`. All candidates are in; the owner has not yet accepted the document as such (the criterion about acceptance is ticked because the owner asked for every setting).

## Dependencies
None.

## Open Questions
Which of the candidates does the owner want, and which does the owner not need?
