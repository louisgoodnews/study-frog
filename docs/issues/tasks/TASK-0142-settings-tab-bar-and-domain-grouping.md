---
id: TASK-0142
type: task
title: Settings tab bar and domain grouping
status: ready
epic: EPIC-023
parents: [STORY-0060]
estimate: 3
labels: [m10, ui]
acceptance_criteria:
  - "Settings show a tab bar and one tab at a time, each with the settings of its domain"
  - "The last tab is remembered per profile and Left and Right move between tabs"
  - "No setting is lost or duplicated, shown by a test that lists every setting once"
  - "The accessibility audit passes"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Split `view/settings.rs` into one view per tab and add the tab bar.

## Acceptance Criteria
- [ ] Settings show a tab bar and one tab at a time, each with the settings of its domain
- [ ] The last tab is remembered per profile and Left and Right move between tabs
- [ ] No setting is lost or duplicated, shown by a test that lists every setting once
- [ ] The accessibility audit passes

## Design Notes
None.

## Dependencies
None.

## Open Questions
None.
