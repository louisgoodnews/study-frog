---
id: TASK-0185
type: task
title: New items default to the highest difficulty and medium priority
status: done
epic: EPIC-023
parents: [STORY-0071]
estimate: 1
labels: [m10, ui]
acceptance_criteria:
  - "An item created without a priority or difficulty gets priority medium and difficulty difficult, written to its file"
  - "A priority or difficulty the user chose is kept"
---

## Context
The owner asked on 2026-11-02 (during the 0.3.0 work) to make the Create dialog easier to use, closer to Jira's Create dialog, and to give new items defaults.

## Description
Cards, notes and questions made in the Create dialog, by import or by the card form write `priority: medium` and `difficulty: difficult` unless the user gave them.

## Acceptance Criteria
- [x] An item created without a priority or difficulty gets priority medium and difficulty difficult, written to its file
- [x] A priority or difficulty the user chose is kept

## Design Notes
None.

## Dependencies
None.

## Open Questions
None.
