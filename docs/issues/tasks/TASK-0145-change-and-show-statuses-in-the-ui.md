---
id: TASK-0145
type: task
title: Change and show statuses in the UI
status: done
epic: EPIC-023
parents: [STORY-0061]
estimate: 3
labels: [m10, ui]
acceptance_criteria:
  - "A status chip on the item page changes the status from a menu"
  - "Lists and cards show the status with its label"
  - "A stack shows how many of its items are in each status"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Add the chip, the menu and the roll-up, with labels from the spec.

## Acceptance Criteria
- [x] A status chip on the item page changes the status from a menu
- [x] Lists and cards show the status with its label
- [x] A stack shows how many of its items are in each status

## Design Notes
The item page has a drop-down menu "Item status" (Automatic and the five statuses) beside priority and difficulty; saving writes the `status` key only when it changed, so an unknown word in a file stays until the user picks a status. List rows show the status label. A stack's roll-up (`status_counts` in the store, `Effect::LoadStatusCounts`) counts its items and those of its sub-stacks by the effective status. Changing a status from a list and on a stack's Board tab comes with the library (STORY-0057) and the stack overview (STORY-0064). Not seen in a real window; tested headless with the accessibility audit.

## Dependencies
The data model task of this story.

## Open Questions
None.
