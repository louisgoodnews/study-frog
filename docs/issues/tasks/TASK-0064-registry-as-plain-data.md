---
id: TASK-0064
type: task
title: Registry as plain data
status: done
epic: EPIC-016
parents: [STORY-0030]
estimate: 2
labels: [m8, ext]
acceptance_criteria:
  - "`Registry` is two Vecs; registration refuses malformed or repeated ids and importers without extensions"
  - "Importers are found by file extension, first registered wins"
---

## Context


## Description
Registry as plain data.

## Acceptance Criteria
- [x] `Registry` is two Vecs; registration refuses malformed or repeated ids and importers without extensions
- [x] Importers are found by file extension, first registered wins

## Design Notes
`studyfrog-ext::registry`, property-tested.

## Dependencies
TASK-0063.

## Open Questions
None.
