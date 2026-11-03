---
id: TASK-0223
type: task
title: Appearance tab in Settings
status: done
epic: EPIC-027
parents: [STORY-0081]
estimate: 3
labels: [m11, ui]
acceptance_criteria:
  - "Settings has an Appearance tab with the theme and the other settings of TASK-0221, a sample that shows the changes live and a button that restores the defaults"
  - "Every control has an accessible name and a keyboard route"
  - "The tab is remembered like the others"
---

## Context
The owner asked on 2026-11-03 for a dark and a light mode and for UI settings in the application, and for an epic for them.

## Description
Where the user finds the settings.

## Acceptance Criteria
- [x] Settings has an Appearance tab with the theme and the other settings of TASK-0221, a sample that shows the changes live and a button that restores the defaults
- [x] Every control has an accessible name and a keyboard route
- [x] The tab is remembered like the others

## Design Notes
Uses the tab bar and the grouped cards of the calmer UI.

Built on 2026-11-03: `view/settings_appearance.rs`: groups for theme, size and density, colours and motion, start-up and window, a live sample and a restore button; accessible names are tested.

## Dependencies
TASK-0221.

## Open Questions
None.
