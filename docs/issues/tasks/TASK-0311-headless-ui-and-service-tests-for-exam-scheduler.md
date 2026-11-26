---
id: TASK-0311
type: task
title: "Headless UI, service, and re-balancing tests for the Exam Scheduler"
status: backlog
epic: EPIC-076
parents: ['STORY-0248']
estimate: 2
labels: [m14, calendar, ui]
acceptance_criteria:
  - 'Service integration tests verify creating an exam plan, attaching stacks/items, proposing weeks from stats, generating blocks, and re-balancing after missed days'
  - 'Headless egui_kittest tests verify the Exam Scheduler UI (Custom vs Proposed toggle, preview rendering, Apply, Calendar/Home visibility, and accessible widget names)'
---

## Context
Part of `STORY-0248`. Provides end-to-end automated verification for the Exam Scheduler.

## Description
Add service integration tests in `crates/studyfrog-service/tests/` and headless UI tests in `crates/studyfrog-ui/tests/` covering the full exam scheduling and re-balancing workflow.

## Acceptance Criteria
- [ ] Service integration tests verify creating an exam plan, attaching stacks/items, proposing weeks from stats, generating blocks, and re-balancing after missed days
- [ ] Headless `egui_kittest` tests verify the Exam Scheduler UI (Custom vs Proposed toggle, preview rendering, Apply, Calendar/Home visibility, and accessible widget names)

## Design Notes
- Run with `RUST_MIN_STACK=16777216` and verify `assert_all_named`.

## Dependencies
- `STORY-0248`, `TASK-0310`

## Open Questions
None.
