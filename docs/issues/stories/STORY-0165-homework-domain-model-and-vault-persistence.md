---
id: STORY-0165
type: story
title: "Homework Data Model, Lifecycle, and Vault Persistence"
status: done
epic: EPIC-056
parents: ['EPIC-056']
estimate: 5
labels: [m14, calendar, store]
acceptance_criteria:
  - 'Pure Rust data model for HomeworkItem with validation in crates/studyfrog-calendar'
  - 'Support for title, course/subject ID, due date, start date, estimated minutes, priority, checklist, and notes'
  - 'Deterministic serialization to vault storage (.studyfrog/homework.toml or stack-relative frontmatter)'
---

## Context
Homework items require a formal domain representation with strict validation and vault-backed persistence before they can be scheduled or rendered in the UI.

## Description
Implement the core data structures and serialization logic for homework items:
- Define `HomeworkItem`, `HomeworkPriority`, and `HomeworkStatus` (`NotStarted`, `InProgress`, `Completed`).
- Provide pure validation functions ensuring non-empty titles, valid date/time bounds, and positive duration estimates.
- Support file-based persistence inside the vault maintaining compatibility with the existing local-first storage architecture.

## Acceptance Criteria
- [x] Pure Rust data model for HomeworkItem with validation in crates/studyfrog-calendar
- [x] Support for title, course/subject ID, due date, start date, estimated minutes, priority, checklist, and notes
- [x] Deterministic serialization to vault storage (.studyfrog/homework.toml or stack-relative frontmatter)

## Design Notes
Keep all types pure with zero inherent `impl` blocks (Invariant P1) and maximum 250 LOC per file (Invariant P6).

## Dependencies
EPIC-056

## Open Questions
None.
