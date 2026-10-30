---
id: TASK-0024
type: task
title: Card and question section split
status: done
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
- [x] Body splits on the first line that is exactly `???`
- [x] A `???` inside a code fence does not split
- [x] Missing separator is a typed error

## Design Notes
`sections.rs` + `codeblock.rs` (shared fence tracker). CRLF tolerated. A front with an unclosed fence cannot be rendered (see TASK-0022).

## Dependencies
TASK-0021.

## Open Questions
None.
