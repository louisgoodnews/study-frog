---
id: TASK-0036
type: task
title: eframe window and navigation
status: ready
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
First dependency on egui/eframe (versions pinned at first use).

## Description
eframe window and navigation.

## Acceptance Criteria
- [ ] Window opens, screen navigation works, vault path chosen and remembered
- [ ] Accessibility/IME spike notes recorded in ADR 0002
- [ ] studyfrog-app depends only on studyfrog-ui

## Design Notes
Keep rendering functions free: `fn screen(state:&State, ui:&mut Ui) -> Vec<Msg>`.

## Dependencies
None.

## Open Questions
Can the app be exercised headlessly in CI (xvfb)? Decide here.
