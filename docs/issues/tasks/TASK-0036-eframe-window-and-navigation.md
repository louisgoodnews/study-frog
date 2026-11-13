---
id: TASK-0036
type: task
title: "eframe window and navigation"
status: done
epic: EPIC-008
parents: [STORY-0016]
estimate: 3
labels: [m4, ui]
acceptance_criteria:
  - "Window opens, screen navigation works, vault path chosen and remembered"
  - "Accessibility/IME spike notes recorded in ADR 0002"
  - "studyfrog-app depends only on studyfrog-ui"
---

## Context
eframe/egui 0.37 (glow, wayland, x11, accesskit); egui_kittest for headless tests. The vault path is a text field plus a recent-vaults list; a native folder dialog would need `rfd` (portal/GTK weight) and is deferred. The last vault is remembered in `$XDG_CONFIG_HOME/studyfrog/app.toml` (`studyfrog-config` + `studyfrog-store`, written atomically).

## Description
eframe window and navigation.

## Acceptance Criteria
- [x] Window opens, screen navigation works, vault path chosen and remembered
- [x] Accessibility/IME spike notes recorded in ADR 0002
- [x] studyfrog-app depends only on studyfrog-ui

## Design Notes
Keep rendering functions free: `fn screen(state:&State, ui:&mut Ui) -> Vec<Msg>`.

## Dependencies
None.

## Open Questions
Resolved: yes. Logic and views are tested headlessly with egui_kittest; a real-window smoke test is `studyfrog --smoke-frames N`, which passed locally (see ADR 0002 spike results).
