---
id: TASK-0118
type: task
title: Detect scanned pages and say so
status: backlog
epic: EPIC-022
parents: [STORY-0054]
estimate: 2
labels: [m9, pdf]
acceptance_criteria:
  - "A page with no extractable text is marked as an image-only page in the viewer and the index"
  - "Extraction and search say that such pages have no text instead of silently finding nothing"
  - "The docs state that OCR is not part of this story"
---

## Context
The owner asked for PDF support on 2026-10-31: PDF files are integral to how StudyFrog is used, but today it cannot open them.

## Description
A cheap check on the extracted text and the page's pictures.

## Acceptance Criteria
- [ ] A page with no extractable text is marked as an image-only page in the viewer and the index
- [ ] Extraction and search say that such pages have no text instead of silently finding nothing
- [ ] The docs state that OCR is not part of this story

## Design Notes
OCR is a separate decision (see the story's open questions).

## Dependencies
TASK-0110.

## Open Questions
None.
