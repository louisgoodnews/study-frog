---
id: TASK-0027
type: task
title: Vault scanner
status: done
epic: EPIC-006
parents: [STORY-0012]
estimate: 5
labels: [m2, store, md]
acceptance_criteria:
  - "Walks the vault, parses every `.md`, returns items plus per-file errors"
  - "Corrupt files reported, not fatal"
  - "Duplicate ids reported on both files"
---

## Context
Single crate doing file I/O.

## Description
Vault scanner.

## Acceptance Criteria
- [x] Walks the vault, parses every `.md`, returns items plus per-file errors
- [x] Corrupt files reported, not fatal
- [x] Duplicate ids reported on both files

## Design Notes
Sequential: 10k notes parse in ~280 ms, so no parallelism. Stack ownership by nearest `_stack.md` folder; inbox stack (uid 0) for orphans; spec: docs/specs/storage-schema.md.

## Dependencies
TASK-0020, TASK-0021, TASK-0025.

## Open Questions
None.
