---
id: STORY-0002
type: story
title: Guardrail scripts
status: done
epic: EPIC-001
parents: [EPIC-001]
estimate: 5
labels: [m0, tooling]
acceptance_criteria:
  - "`scripts/check.sh` runs fmt, clippy, test, line count, layering, impl scan"
  - "Each check demonstrably fails on a violation"
---

## Context
P1, P6, P8 need enforcement beyond review.

## Description
Guardrail scripts for M0.

## Acceptance Criteria
- [x] `scripts/check.sh` runs fmt, clippy, test, line count, layering, impl scan
- [x] Each check demonstrably fails on a violation

## Design Notes
Children: TASK-0004, TASK-0005, TASK-0006.

## Dependencies
Parent epic EPIC-001.

## Open Questions
None.
