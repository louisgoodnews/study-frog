---
id: TASK-0012
type: task
title: "Id newtypes and arena indices"
status: done
epic: EPIC-003
parents: [STORY-0005]
estimate: 3
labels: [m1, core]
acceptance_criteria:
  - "`Uid` (128-bit stable id) and `Idx<T>` (typed u32 arena index) are data-only types"
  - "Generic `Arena<T>` with free functions: push, get, get_mut, len, iter"
  - "Idx is Copy/Eq/Ord/Hash for every T (no T bounds)"
  - "proptests: push/get round-trip, indices sequential, ordering matches raw"
  - "No inherent impl blocks; files under 150 lines"
---

## Context
`Uid` is the ULID bits persisted in front-matter (parsing and formatting belong to studyfrog-md). `Idx<T>` is an in-memory handle into an `Arena<T>`; it is never persisted.

## Description
Id newtypes and arena indices.

## Acceptance Criteria
- [x] `Uid` (128-bit stable id) and `Idx<T>` (typed u32 arena index) are data-only types
- [x] Generic `Arena<T>` with free functions: push, get, get_mut, len, iter
- [x] Idx is Copy/Eq/Ord/Hash for every T (no T bounds)
- [x] proptests: push/get round-trip, indices sequential, ordering matches raw
- [x] No inherent impl blocks; files under 150 lines

## Design Notes
Manual trait impls for `Idx<T>` (derive would add `T: Trait` bounds); each body delegates to the raw index. `PhantomData<fn() -> T>` keeps Idx Send/Sync regardless of T.

## Dependencies
None.

## Open Questions
Should Idx carry a generation counter once deletion exists? Defer until an entity needs removal.
