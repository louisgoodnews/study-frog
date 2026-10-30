---
id: TASK-0006
type: task
title: impl-block scan for P1
status: done
epic: EPIC-001
parents: [STORY-0002]
estimate: 2
labels: [m0, tooling]
acceptance_criteria:
  - "`scripts/check-impls.sh` flags inherent impl blocks"
  - "Trait impls pass; `// allow-impl: reason` exempts one block"
---

## Context
Part of STORY-0002.

## Description
impl-block scan for P1.

## Acceptance Criteria
- [x] `scripts/check-impls.sh` flags inherent impl blocks
- [x] Trait impls pass; `// allow-impl: reason` exempts one block

## Design Notes
Line-based awk scan; known limit: misses impls split across lines or in macros.

## Dependencies
Parent story STORY-0002.

## Open Questions
Replace with a syn-based check if false negatives appear.
