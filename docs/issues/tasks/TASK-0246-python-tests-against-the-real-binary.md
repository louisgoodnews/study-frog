---
id: TASK-0246
type: task
title: "Python tests against the real binary"
status: done
epic: EPIC-028
parents: [STORY-0131]
estimate: 2
labels: [m12, app]
acceptance_criteria:
  - "`python/tests/` unittest suite run by `crates/studyfrog-app/tests/python_client.rs` against a headless instance in a temporary home"
---
## Context
Part of STORY-0131.

## Description
See the story and ADR 0013.

## Acceptance Criteria
- [x] `python/tests/` unittest suite run by `crates/studyfrog-app/tests/python_client.rs` against a headless instance in a temporary home

## Design Notes
See the story.

## Dependencies
STORY-0131.

## Open Questions
None.
