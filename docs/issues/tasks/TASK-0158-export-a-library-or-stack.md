---
id: TASK-0158
type: task
title: "Export a library or stack"
status: done
epic: EPIC-023
parents: [STORY-0065]
estimate: 3
labels: [m10, app, md]
acceptance_criteria:
  - "JSON, CSV and Markdown exports of the library or of one stack, with pictures and PDFs in a zip"
  - "Exporting and importing a JSON file into an empty vault gives the same items, verified by a round-trip test"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Add pure encoders and the service function that writes them.

## Acceptance Criteria
- [x] JSON, CSV and Markdown exports of the library or of one stack, with pictures and PDFs in a zip
- [x] Exporting and importing a JSON file into an empty vault gives the same items, verified by a round-trip test

## Design Notes
None.

## Dependencies
The specification task of this story.

## Open Questions
None.
