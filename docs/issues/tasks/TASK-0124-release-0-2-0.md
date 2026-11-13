---
id: TASK-0124
type: task
title: "Release 0.2.0"
status: done
epic: EPIC-022
parents: [STORY-0054]
estimate: 2
labels: [m9, pdf]
acceptance_criteria:
  - "The workspace version is 0.2.0 and the changelog has a 0.2.0 section dated the release day"
  - "Release notes for 0.2.0 exist where the release workflow looks for them"
  - "The work is pushed to origin, tagged v0.2.0, and the GitHub release carries the AppImage and its SHA-256"
---

## Context
The owner asked on 2026-10-31 for links inside a PDF to be followed with the user's approval, and for the PDF feature to be released as version 0.2.0.

## Description
Version bump, notes, push and tag; the release workflow builds and publishes.

## Acceptance Criteria
- [x] The workspace version is 0.2.0 and the changelog has a 0.2.0 section dated the release day
- [x] Release notes for 0.2.0 exist where the release workflow looks for them
- [x] The work is pushed to origin, tagged v0.2.0, and the GitHub release carries the AppImage and its SHA-256

## Design Notes
The notes say what was checked by hand and what only by tests.

## Dependencies
TASK-0112.

## Open Questions
None.
