---
id: STORY-0078
type: story
title: Imports that explain themselves
status: done
epic: EPIC-026
parents: [EPIC-026]
estimate: 5
labels: [m11, ui, store, docs]
acceptance_criteria:
  - "CSV files with other column layouts can be imported by mapping their columns"
  - "The import dialog says which layout and format each importer reads, with an example"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports.

## Description
Make importing less of a guess: tell the user what is expected and cope with the files people really have.

## Acceptance Criteria
- [x] CSV files with other column layouts can be imported by mapping their columns
- [x] The import dialog says which layout and format each importer reads, with an example

## Design Notes
Tasks TASK-0208 and TASK-0209.

## Dependencies
None.

## Open Questions
None.
