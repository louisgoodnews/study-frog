---
id: TASK-0168
type: task
title: Obsidian vault importer
status: done
epic: EPIC-023
parents: [STORY-0067]
estimate: 3
labels: [m10, app, ext, md]
acceptance_criteria:
  - "Folders become stacks, Markdown files become notes with their front-matter, wikilinks, tags and pictures kept"
  - "Syntax StudyFrog does not support is kept as text and counted in the preview"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Write the adapter for an Obsidian folder.

## Acceptance Criteria
- [x] Folders become stacks, Markdown files become notes with their front-matter, wikilinks, tags and pictures kept
- [x] Syntax StudyFrog does not support is kept as text and counted in the preview

## Design Notes
None.

## Dependencies
The import engine task of STORY-0065.

## Open Questions
None.
