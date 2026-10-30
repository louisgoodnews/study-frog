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
Implemented with syn in `tools/repo-lint` (v1 was a line-based awk scan; replaced 2026-10-30). Detects inherent impls anywhere in the AST and in macro bodies (heuristic: `impl` token without `for`). Exemption marker: `// allow-impl: reason` on the line above.

## Dependencies
Parent story STORY-0002.

## Open Questions
Macro detection is token-level and heuristic; revisit if a macro-heavy crate appears.
