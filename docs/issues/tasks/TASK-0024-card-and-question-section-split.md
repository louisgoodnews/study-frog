---
id: TASK-0024
type: task
title: Card and question section split
status: backlog
epic: EPIC-005
parents: [STORY-0010]
estimate: 2
labels: [m2, md]
acceptance_criteria:
  - "Body splits on the first line that is exactly `???`"
  - "A `???` inside a code fence does not split"
  - "Missing separator is a typed error"
---

## Context
Spec: docs/specs/markdown-schema.md.

## Description
Card and question section split.

## Acceptance Criteria
- [ ] Body splits on the first line that is exactly `???`
- [ ] A `???` inside a code fence does not split
- [ ] Missing separator is a typed error

## Design Notes
Fence-aware line scan.

## Dependencies
TASK-0021.

## Open Questions
None.
