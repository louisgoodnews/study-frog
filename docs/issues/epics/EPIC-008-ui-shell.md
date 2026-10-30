---
id: EPIC-008
type: epic
title: UI shell
status: done
parents: []
labels: [m4, ui]
acceptance_criteria:
  - "App launches and opens a vault"
  - "User can create a stack, write a card, choose how to rehearse (any order, filters, schedule effect), rehearse with the keyboard and see it rescheduled"
  - "`update` layer is pure and unit-tested; the ui crate holds no business logic"
---

## Context
First usable desktop app (ADR 0002: egui/eframe with our own pure update layer).

## Description
egui/eframe window, navigation, Elm-style `update(State, Msg) -> (State, Effects)`, stack browser, run setup and rehearsal screens.

## Acceptance Criteria
- [x] App launches and opens a vault
- [x] User can create a stack, write a card, choose how to rehearse (any order, filters, schedule effect), rehearse with the keyboard and see it rescheduled
- [x] `update` layer is pure and unit-tested; the ui crate holds no business logic

## Design Notes
Children: STORY-0016, STORY-0017.

## Dependencies
M3 exit; ADR 0002 accepted; studyfrog-service (ADR 0006).

## Open Questions
Accessibility and IME in egui: spike in TASK-0036; fallback is iced (ADR 0002).
