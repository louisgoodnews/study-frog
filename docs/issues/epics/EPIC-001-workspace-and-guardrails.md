---
id: EPIC-001
type: epic
title: Workspace and guardrails
status: done
parents: []
labels: [m0, tooling]
acceptance_criteria:
  - "`cargo check --workspace` passes"
  - "`scripts/check.sh` enforces fmt, clippy, tests, size, layering, no-OOP"
---

## Context
Constraints are non-negotiable and must be machine-checked where possible.

## Description
Compilable workspace and automated guardrails enforcing the engineering constraints.

## Acceptance Criteria
- [x] `cargo check --workspace` passes
- [x] `scripts/check.sh` enforces fmt, clippy, tests, size, layering, no-OOP

## Design Notes
Children: STORY-0001, STORY-0002.

## Dependencies
None; first epic.

## Open Questions
None.
