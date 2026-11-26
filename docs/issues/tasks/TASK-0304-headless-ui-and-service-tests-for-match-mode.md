---
id: TASK-0304
type: task
title: "Headless UI and service tests for vault-sourced Match mode"
status: done
epic: EPIC-075
parents: ['STORY-0245']
estimate: 1
labels: [m14, ui, service]
acceptance_criteria:
  - 'Service tests verify Card and Question pair extraction, Markdown stripping, stack filtering, and fresh-round sampling'
  - 'Headless egui_kittest tests verify real vault pairs render in Match mode (never biology demo cards), scope switching, New round, empty state, and accessible widget names'
---

## Context
Part of `STORY-0245`. Ensures vault-sourced Match mode cannot regress back to hardcoded demo cards.

## Description
Add unit/integration tests in `crates/studyfrog-service/tests/` and `crates/studyfrog-ui/tests/` covering pair extraction, scope filtering, empty state, and interactive tile matching.

## Acceptance Criteria
- [x] Service tests verify Card and Question pair extraction, Markdown stripping, stack filtering, and fresh-round sampling
- [x] Headless `egui_kittest` tests verify real vault pairs render in Match mode (never biology demo cards), scope switching, New round, empty state, and accessible widget names

## Design Notes
- Run with `RUST_MIN_STACK=16777216` and verify `assert_all_named`.

## Dependencies
- `STORY-0245`, `TASK-0303`

## Open Questions
None.
