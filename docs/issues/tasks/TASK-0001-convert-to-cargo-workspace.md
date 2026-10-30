---
id: TASK-0001
type: task
title: Convert to Cargo workspace
status: done
epic: EPIC-001
parents: [STORY-0001]
estimate: 1
labels: [m0, tooling]
acceptance_criteria:
  - "Root manifest is a virtual workspace with shared deps, package metadata and lints"
  - "Hello-world package removed"
---

## Context
Part of STORY-0001.

## Description
Convert to Cargo workspace.

## Acceptance Criteria
- [x] Root manifest is a virtual workspace with shared deps, package metadata and lints
- [x] Hello-world package removed

## Design Notes
resolver 3, edition 2024, `[workspace.dependencies]` lists internal crates; third-party versions pinned at first use.

## Dependencies
Parent story STORY-0001.

## Open Questions
None.
