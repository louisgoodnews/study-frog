---
id: TASK-0211
type: task
title: Stack file format, reading and migration
status: backlog
epic: EPIC-026
parents: [STORY-0079]
estimate: 8
labels: [m11, md, store]
acceptance_criteria:
  - "Stack files with their contents are read and written, and unknown keys are kept as now"
  - "A vault in the old layout (a folder with `_stack.md`) is read as before and a migration writes the new layout after a backup, which can be undone"
  - "The round-trip property test covers the new format"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports. The owner answered the open questions the same day (recorded below).

## Description
The parser, the scanner and the index for the new stack format, and the migration of existing vaults.

## Acceptance Criteria
- [ ] Stack files with their contents are read and written, and unknown keys are kept as now
- [ ] A vault in the old layout (a folder with `_stack.md`) is read as before and a migration writes the new layout after a backup, which can be undone
- [ ] The round-trip property test covers the new format

## Design Notes
Follows the ADR of TASK-0210. The index keeps a membership table (stack, item, position) instead of deriving it from folders; the scanner builds it from the stack files; a schema migration adds it.

## Dependencies
TASK-0210.

## Open Questions
None.
