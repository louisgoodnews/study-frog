---
id: TASK-0244
type: task
title: Event bus and the subscribe command
status: done
epic: EPIC-028
parents: [STORY-0130]
estimate: 3
labels: [m12, ui, app]
acceptance_criteria:
  - "`subscribe [kinds...]` registers the connection on the bus and streams matching events until it closes; tested over in-memory streams, a socket and the binary"
---
## Context
Part of STORY-0130.

## Description
See the story and ADR 0013.

## Acceptance Criteria
- [x] `subscribe [kinds...]` registers the connection on the bus and streams matching events until it closes; tested over in-memory streams, a socket and the binary

## Design Notes
See the story.

## Dependencies
STORY-0130.

## Open Questions
None.
