---
id: TASK-0222
type: task
title: "Choose the theme (dark, light or follow the system)"
status: done
epic: EPIC-027
parents: [STORY-0080]
estimate: 3
labels: [m11, ui]
acceptance_criteria:
  - "A `theme` setting (dark, light or system) is read from and written to app.toml and the first start uses the system's"
  - "The choice is changed in the application (a control in the top bar's profile menu and in Settings) and takes effect at once without restarting"
  - "With 'system' a change of the system's theme while the app runs is followed"
  - "The choice is tested: the visuals in egui's context follow the setting"
---

## Context
The owner asked on 2026-11-03 for a dark and a light mode and for UI settings in the application, and for an epic for them.

## Description
The palettes of both themes exist; the application follows the system without a choice today.

## Acceptance Criteria
- [x] A `theme` setting (dark, light or system) is read from and written to app.toml and the first start uses the system's
- [x] The choice is changed in the application (a control in the top bar's profile menu and in Settings) and takes effect at once without restarting
- [x] With 'system' a change of the system's theme while the app runs is followed
- [x] The choice is tested: the visuals in egui's context follow the setting

## Design Notes
`theme::apply` already installs the visuals for both egui themes; add `ThemePreference` to `AppSettings` and call `ctx.set_theme(...)` from an effect or at start-up. Pure state: the setting; the context change in the shell.

Built on 2026-11-03: `theme` (system, dark, light) in `app.toml`; the control is in Settings > Appearance and the profile menu; `theme_apply::sync` applies it every frame a setting changed, so no restart is needed; the first start follows the system. Tested through the egui context (`tests/appearance.rs`).

## Dependencies
TASK-0221.

## Open Questions
None.
