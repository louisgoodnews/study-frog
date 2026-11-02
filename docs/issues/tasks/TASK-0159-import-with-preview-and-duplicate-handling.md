---
id: TASK-0159
type: task
title: Import with preview and duplicate handling
status: done
epic: EPIC-023
parents: [STORY-0065]
estimate: 5
labels: [m10, app, ext]
acceptance_criteria:
  - "A dry run reports creates, duplicates and errors without writing"
  - "The user chooses to skip, replace or keep both for duplicates, and nothing is written on a cancel"
  - "Malformed and hostile files cannot crash the app, shown by a corpus test"
  - "The existing TSV import runs through the same path"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Add decoders and the dry-run engine and register the formats in the extension registry.

## Acceptance Criteria
- [x] A dry run reports creates, duplicates and errors without writing
- [x] The user chooses to skip, replace or keep both for duplicates, and nothing is written on a cancel
- [x] Malformed and hostile files cannot crash the app, shown by a corpus test
- [x] The existing TSV import runs through the same path

## Design Notes
None.

## Dependencies
The specification task of this story.

## Open Questions
None.
