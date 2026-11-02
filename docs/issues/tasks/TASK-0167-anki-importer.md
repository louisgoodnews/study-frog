---
id: TASK-0167
type: task
title: Anki importer
status: backlog
epic: EPIC-023
parents: [STORY-0067]
estimate: 5
labels: [m10, app, ext]
acceptance_criteria:
  - "Decks become stacks, basic and cloze notes become cards, and tags and pictures are kept"
  - "Review history is imported as reviews and the schedule is adapted from it, with a checkbox to leave it out"
  - "A corpus of broken and hostile .apkg files neither crashes nor hangs the app"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Write the adapter: zip, SQLite collection, media map, mapping to the interchange model.

## Acceptance Criteria
- [ ] Decks become stacks, basic and cloze notes become cards, and tags and pictures are kept
- [ ] Review history is imported as reviews and the schedule is adapted from it, with a checkbox to leave it out
- [ ] A corpus of broken and hostile .apkg files neither crashes nor hangs the app

## Design Notes
None.

## Dependencies
The import engine task of STORY-0065.

## Open Questions
None.
