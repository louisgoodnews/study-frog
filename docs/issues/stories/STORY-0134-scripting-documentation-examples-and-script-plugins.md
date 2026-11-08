---
id: STORY-0134
type: story
title: Scripting documentation, examples and script plugins
status: backlog
epic: EPIC-028
parents: [EPIC-028]
estimate: 3
labels: [m12, docs]
acceptance_criteria:
  - "A help page and a spec describe the Python API, scripts.toml, events, access levels and the console; `examples/scripts/` holds a daily vault health check, an export script, an event-bound script and a script importer, each run by a test"
  - "docs/EXTENDING.md says what the compiled-in registry offers (importers, card types, format adapters) and what scripts add out of process, and what is not possible"
---
## Context
Users need to know how to write and install scripts (EPIC-028).

## Description
Help topic `scripting`, spec `docs/specs/scripting.md`, examples, EXTENDING.md section. Tasks: TASK-0251, TASK-0252.

## Acceptance Criteria
- [ ] A help page and a spec describe the Python API, scripts.toml, events, access levels and the console; `examples/scripts/` holds a daily vault health check, an export script, an event-bound script and a script importer, each run by a test
- [ ] docs/EXTENDING.md says what the compiled-in registry offers (importers, card types, format adapters) and what scripts add out of process, and what is not possible

## Design Notes
ADR 0013 (`docs/adr/0013-scripting-out-of-process.md`).

## Dependencies
STORY-0131, STORY-0132, STORY-0133.

## Open Questions
None.
