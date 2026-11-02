---
id: TASK-0144
type: task
title: Status field in the data model and Markdown schema
status: done
epic: EPIC-023
parents: [STORY-0061]
estimate: 3
labels: [m10, core, md, store]
acceptance_criteria:
  - "Every item kind has a status that is written to and read from the front-matter and survives a round trip"
  - "Items without a status read as 'new' and are derived to 'learning' after their first review"
  - "The index stores the status so lists can sort and filter by it"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Add the field, the schema key, the default and the index column with a migration.

## Acceptance Criteria
- [x] Every item kind has a status that is written to and read from the front-matter and survives a round trip
- [x] Items without a status read as 'new' and are derived to 'learning' after their first review
- [x] The index stores the status so lists can sort and filter by it

## Design Notes
`studyfrog_core::status` holds `Status`, the stored words, the labels, `effective_status` and `skips_schedule`; `studyfrog_md::status` reads and writes the `status` key on every kind (an unknown word reads as none and stays in the file until the user sets a status); the index has an `item.status` column (migration 12) and every item list (`ItemRow`) carries the effective status, derived in the store from the stored word and whether a review exists. The first review does not rewrite the file.

## Dependencies
The specification task of this story.

## Open Questions
None.
