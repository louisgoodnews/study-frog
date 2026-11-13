---
id: TASK-0196
type: task
title: "Navigation history"
status: done
epic: EPIC-024
parents: [STORY-0073]
estimate: 5
labels: [m11, ui]
acceptance_criteria:
  - "Screens, stacks and items visited are noted in order, without repeats, and kept for the session; a new vault starts a new history (at most 50 places)"
  - "Back and Forward buttons and a History list in the top bar, and Alt+Left and Alt+Right, go to the places; travelling adds no new places and a new visit after Back drops the places ahead"
  - "Unsaved changes in the editor block leaving and say why"
---

## Context
The owner sent this after trying 0.3.1 on 2026-11-03.

## Description
A browser-like history with in-UI access.

## Acceptance Criteria
- [x] Screens, stacks and items visited are noted in order, without repeats, and kept for the session; a new vault starts a new history (at most 50 places)
- [x] Back and Forward buttons and a History list in the top bar, and Alt+Left and Alt+Right, go to the places; travelling adds no new places and a new visit after Back drops the places ahead
- [x] Unsaved changes in the editor block leaving and say why

## Design Notes
`history.rs` (pure): a place is a screen, a stack's folder or an item's uid; `hooks::around` compares the place before and after every update and notes a change, so no screen reports anything. A jump sets `skip` so that arriving is not a new visit. Item titles are kept beside the places. The history is not saved across restarts (a proposal if wanted). Alt+arrow no longer switches tabs.

## Dependencies
None.

## Open Questions
None.
