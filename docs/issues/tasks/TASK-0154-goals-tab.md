---
id: TASK-0154
type: task
title: Goals tab
status: done
epic: EPIC-023
parents: [STORY-0064]
estimate: 2
labels: [m10, ui, md]
acceptance_criteria:
  - "The tab shows the stack's goals as a bullet list that the user edits in place at any time, with check boxes allowed"
  - "Goals are stored under a Goals heading in the stack's Markdown file and survive a round trip, a restart and a backup"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Add the Goals section to the stack file and an editor on the tab.

## Acceptance Criteria
- [x] The tab shows the stack's goals as a bullet list that the user edits in place at any time, with check boxes allowed
- [x] Goals are stored under a Goals heading in the stack's Markdown file and survive a round trip, a restart and a backup

## Design Notes
None.

## Dependencies
STORY-0063.

## Open Questions
None.
