---
id: TASK-0119
type: task
title: Extract pictures and page snapshots into the vault
status: done
epic: EPIC-022
parents: [STORY-0054]
estimate: 3
labels: [m9, pdf]
acceptance_criteria:
  - "The pictures on a page can be saved into the vault's assets folder and inserted as Markdown images into a note"
  - "A rectangle of a page can be saved as a picture (for figures and formulas)"
  - "Names and de-duplication follow the existing picture rules and the 20 MB limit"
---

## Context
The owner asked for PDF support on 2026-10-31: PDF files are integral to how StudyFrog is used, but today it cannot open them.

## Description
Reuses service::images and the Markdown image support.

## Acceptance Criteria
- [x] The pictures on a page can be saved into the vault's assets folder and inserted as Markdown images into a note
- [x] A rectangle of a page can be saved as a picture (for figures and formulas)
- [x] Names and de-duplication follow the existing picture rules and the 20 MB limit

## Design Notes
A page snapshot is a render at a chosen scale cropped to the rectangle.

## Dependencies
TASK-0110, TASK-0112.

## Open Questions
None.
