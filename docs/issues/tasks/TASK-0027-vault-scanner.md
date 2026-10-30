---
id: TASK-0027
type: task
title: Vault scanner
status: backlog
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
- [ ] Walks the vault, parses every `.md`, returns items plus per-file errors
- [ ] Corrupt files reported, not fatal
- [ ] Duplicate ids reported on both files

## Design Notes
Parallelism only if the benchmark demands it.

## Dependencies
TASK-0020, TASK-0021, TASK-0025.

## Open Questions
None.
