---
id: TASK-0029
type: task
title: "File watcher"
status: done
epic: EPIC-006
parents: [STORY-0012]
estimate: 3
labels: [m2, store]
acceptance_criteria:
  - "Create/modify/delete/rename events update the index"
  - "Burst of events coalesced"
---

## Context
Keeps the index fresh while the app runs.

## Description
File watcher.

## Acceptance Criteria
- [x] Create/modify/delete/rename events update the index
- [x] Burst of events coalesced

## Design Notes
**Decision (P10): polling, no watcher dependency.** A stat-only poll of 10k files costs ~43 ms, bursts coalesce by construction, files modified within a settle window are deferred so half-written files are not indexed, and the clock is injected. inotify (via the `inotify` crate; our own code is `unsafe`-free) remains an option if a profile ever shows polling matters. `notify` was rejected: cross-platform weight we do not need.

## Dependencies
TASK-0028.

## Open Questions
None.
