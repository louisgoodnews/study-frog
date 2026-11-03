---
id: TASK-0171
type: task
title: Diary entries in the data model, Markdown and index
status: backlog
epic: EPIC-024
parents: [STORY-0068]
estimate: 3
labels: [m11, core, md, store]
acceptance_criteria:
  - "A diary entry reads and writes as Markdown with its ratings, links and pictures and survives a round trip"
  - "The index lists entries by day and finds them by text and by linked item"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Add the entry type, the schema, the scanner support and the index table.

## Acceptance Criteria
- [ ] A diary entry reads and writes as Markdown with its ratings, links and pictures and survives a round trip
- [ ] The index lists entries by day and finds them by text and by linked item

## Design Notes
None.

## Dependencies
The specification task of this story.

## Open Questions
None.
