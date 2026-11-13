---
id: TASK-0075
type: task
title: "Stack settings in the index"
status: done
epic: EPIC-018
parents: [STORY-0034]
estimate: 5
labels: [m3, md, core, store]
acceptance_criteria:
  - "`_stack.toml` (scheduler, request_retention, daily limits) and `studyfrog.toml` are parsed, validated and rendered by the new `studyfrog-config` crate; `StackSettings`/`VaultSettings` live in core"
  - "Migration v3 adds `stack_identity` (from `_stack.md`), `stack_config` (synced from `_stack.toml`) and the view `stack_setting`"
  - "Store can answer: which stack and scheduler does an item use"
  - "Unknown keys are errors with line numbers; render then parse is the identity (property test)"
---

## Context
**Owner decision (2026-10-30): settings live in TOML files, not front-matter** (ADR 0007, Proposed). The service must not re-parse files at record time, so settings are indexed.

## Description
Stack settings in the index.

## Acceptance Criteria
- [x] `_stack.toml` (scheduler, request_retention, daily limits) and `studyfrog.toml` are parsed, validated and rendered by the new `studyfrog-config` crate; `StackSettings`/`VaultSettings` live in core
- [x] Migration v3 adds `stack_identity` (from `_stack.md`), `stack_config` (synced from `_stack.toml`) and the view `stack_setting`
- [x] Store can answer: which stack and scheduler does an item use
- [x] Unknown keys are errors with line numbers; render then parse is the identity (property test)

## Design Notes
`stack_identity` cascades with its `_stack.md`; `stack_config` is keyed by folder and re-read only when the file's size or mtime moves (one `stat` per stack). `poll_changes` includes the settings files in its fingerprint. Invalid files store their error and fall back to the vault defaults. `_stack.toml` without `_stack.md` is ignored.

## Dependencies
TASK-0074 (shares migration v3).

## Open Questions
None.
