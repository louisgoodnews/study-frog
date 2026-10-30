---
id: TASK-0013
type: task
title: Stack, Note, Card, Question structs
status: done
epic: EPIC-003
parents: [STORY-0005]
estimate: 3
labels: [m1, core]
acceptance_criteria:
  - "Plain structs matching docs/specs/markdown-schema.md fields"
  - "Each has a `uid: Uid`; cross references use `Uid` (persisted) or `Idx<T>` (in memory)"
  - "No behaviour blocks"
---

## Context
Item kinds from the glossary.

## Description
Stack, Note, Card, Question structs.

## Acceptance Criteria
- [x] Plain structs matching docs/specs/markdown-schema.md fields
- [x] Each has a `uid: Uid`; cross references use `Uid` (persisted) or `Idx<T>` (in memory)
- [x] No behaviour blocks

## Design Notes
Data only; no parsing here. **Decision (owner, 2026-10-30): separate arenas per kind, no `Item` enum.** `Library` holds `Arena<Stack|Note|Card|Question>`; items point to their stack by `Idx<Stack>`. Common fields live in a `Meta` struct (composition). Timestamps are `UnixMs(i64)` so `core` stays dependency-free.

## Dependencies
TASK-0012.

## Open Questions
None. Remaining per-stack SRS overrides are added with the SRS parameter type (TASK-0016).
