---
id: TASK-0287
type: task
title: "Classroom domain types and pure state transitions in studyfrog-core"
status: done
epic: EPIC-068
parents: ['STORY-0208']
estimate: 2
labels: [m14, core]
acceptance_criteria:
  - 'studyfrog-core defines pure data structs and enums for ClassroomSession, Participant (Teacher, SimulatedStudent, User), ChatTurn, Assignment (Solo, Group), GroupRoster, Submission, and GradeReport'
  - 'Pure free functions validate state transitions (append turn, assign task, submit answer, record grade) and reject invalid operations (e.g. grading before turn-in or editing a submitted answer)'
  - 'Unit tests cover all transitions and edge cases with zero inherent impl blocks and files <= 250 LOC'
---

## Context
Part of `STORY-0208`. All classroom domain types and invariants belong in `studyfrog-core` so storage, agent orchestration, and UI share one typed representation.

## Description
Add a `classroom` module to `studyfrog-core` with plain structs/enums and pure free functions for session creation, turn appending, solo/group assignment validation, submission locking, and grade recording.

## Acceptance Criteria
- [x] `studyfrog-core` defines pure data structs and enums for `ClassroomSession`, `Participant`, `ChatTurn`, `Assignment`, `GroupRoster`, `Submission`, and `GradeReport`
- [x] Pure free functions validate state transitions (append turn, assign task, submit answer, record grade) and reject invalid operations
- [x] Unit tests cover all transitions and edge cases with zero inherent `impl` blocks and files $\le 250$ LOC

## Design Notes
- Follow Invariant P1 (pure free functions) and P6 ($\le 250$ LOC per file).
- Keep all timestamps as `UnixMs` and identifiers as `Uid`.

## Dependencies
- `STORY-0208`

## Open Questions
None.
