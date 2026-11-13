---
id: TASK-0157
type: task
title: "Specify the interchange formats"
status: done
epic: EPIC-023
parents: [STORY-0065]
estimate: 3
labels: [m10, docs, decision]
acceptance_criteria:
  - "docs/specs/interchange.md defines the JSON schema with its version, the CSV columns, the Markdown layout and how duplicates and conflicts are decided"
  - "The spec names Anki and Obsidian as the first importers and says how a new format adapter is added"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Write the formats down before building them.

## Acceptance Criteria
- [x] docs/specs/interchange.md defines the JSON schema with its version, the CSV columns, the Markdown layout and how duplicates and conflicts are decided
- [x] The spec names Anki and Obsidian as the first importers and says how a new format adapter is added

## Design Notes
None.

## Dependencies
None.

## Open Questions
None.
