---
id: TASK-0160
type: task
title: Import and export screens
status: in_progress
epic: EPIC-023
parents: [STORY-0065]
estimate: 3
labels: [m10, ui]
acceptance_criteria:
  - "A dialog picks the format and the scope, shows the preview and the result"
  - "Starting an export opens a modal that asks for the export mode: Default (without scheduling) or Advanced, whose only option for now is to include scheduling (review history and schedule), and export and import are reachable from the Create menu, a stack's menu and Settings under Data"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Build the dialogs on the service functions.

## Acceptance Criteria
- [ ] A dialog picks the format and the scope, shows the preview and the result
- [ ] Starting an export opens a modal that asks for the export mode: Default (without scheduling) or Advanced, whose only option for now is to include scheduling (review history and schedule), and export and import are reachable from the Create menu, a stack's menu and Settings under Data

## Design Notes
None.

## Dependencies
The export and import tasks of this story.

## Open Questions
None.
