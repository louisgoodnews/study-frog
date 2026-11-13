---
id: TASK-0089
type: task
title: "The user marks suggest ratings"
status: done
epic: EPIC-019
parents: [STORY-0035]
estimate: 3
labels: [m6, followup]
acceptance_criteria:
  - "Marked difficult and rated Good or Easy suggests Hard; marked easy and rated Good suggests Easy"
  - "The mark wins over reading speed; low ratings are never softened"
---

## Context
An open flag from STATUS.md.

## Description
The user marks suggest ratings.

## Acceptance Criteria
- [x] Marked difficult and rated Good or Easy suggests Hard; marked easy and rated Good suggests Easy
- [x] The mark wins over reading speed; low ratings are never softened

## Design Notes
`rehearsal::reading::suggest_from_mark`. Marks still do not change the schedule by themselves: applying a suggestion is the user click (TASK-0083).

## Dependencies
None.

## Open Questions
None.
