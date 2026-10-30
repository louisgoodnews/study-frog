---
id: STORY-0001
type: story
title: Workspace skeleton
status: done
epic: EPIC-001
parents: [EPIC-001]
estimate: 3
labels: [m0, core, tooling]
acceptance_criteria:
  - "12 crates exist under `crates/` with lib.rs doc comments"
  - "App depends only on ui; no domain crate depends on ui or app"
  - "Dependency direction is machine-checked"
---

## Context
Layout from ADR 0001.

## Description
Workspace skeleton for M0.

## Acceptance Criteria
- [x] 12 crates exist under `crates/` with lib.rs doc comments
- [x] App depends only on ui; no domain crate depends on ui or app
- [x] Dependency direction is machine-checked

## Design Notes
Children: TASK-0001, TASK-0002, TASK-0003.

## Dependencies
Parent epic EPIC-001.

## Open Questions
None.
