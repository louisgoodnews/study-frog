---
id: TASK-0142
type: task
title: Settings tab bar and domain grouping
status: done
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
- [x] Settings show a tab bar and one tab at a time, each with the settings of its domain
- [x] The last tab is remembered per profile and Left and Right move between tabs
- [x] No setting is lost or duplicated, shown by a test that lists every setting once
- [x] The accessibility audit passes

## Design Notes
The tabs are data: `SettingsTab` and `SHOWN` in `app_settings_ui.rs`, and `sections_of` in `view/settings.rs` maps a tab to its groups of settings. A tab with no settings yet (General, Appearance, Notifications) is not shown, and joins `SHOWN` when its first setting arrives. The last tab is the `settings_tab` key of `app.toml`, which is one file per profile. Left and Right step through the tabs when no text field has focus. Not seen in a real window; tested headless with the accessibility audit.

## Dependencies
None.

## Open Questions
None.
