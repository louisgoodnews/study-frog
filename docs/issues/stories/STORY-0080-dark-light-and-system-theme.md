---
id: STORY-0080
type: story
title: Dark, light and system theme
status: backlog
epic: EPIC-027
parents: [EPIC-027]
estimate: 8
labels: [m11, ui]
acceptance_criteria:
  - "A setting chooses Dark, Light or Follow system, takes effect at once and is remembered"
  - "Every screen, dialog, chart and toast reads well in both themes, checked with screenshots"
  - "The palette meets a contrast rule that a test checks"
---

## Context
The owner asked on 2026-11-03 for a dark and a light mode and for UI settings in the application, and for an epic for them.

## Description
The palettes for both themes exist in `theme.rs`, but the user cannot choose and the screens were only looked at in dark.

## Acceptance Criteria
- [ ] A setting chooses Dark, Light or Follow system, takes effect at once and is remembered
- [ ] Every screen, dialog, chart and toast reads well in both themes, checked with screenshots
- [ ] The palette meets a contrast rule that a test checks

## Design Notes
Tasks TASK-0221, TASK-0222 and TASK-0226.

## Dependencies
None.

## Open Questions
None.
