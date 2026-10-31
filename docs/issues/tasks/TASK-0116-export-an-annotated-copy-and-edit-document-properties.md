---
id: TASK-0116
type: task
title: Export an annotated copy and edit document properties
status: backlog
epic: EPIC-022
parents: [STORY-0054]
estimate: 3
labels: [m9, pdf]
acceptance_criteria:
  - "'Save a copy with annotations' writes highlights and notes into a new PDF that other readers show"
  - "Title, author and subject can be edited and are written to the copy"
  - "The sidecar is unchanged and the original is untouched"
---

## Context
The owner asked for PDF support on 2026-10-31: PDF files are integral to how StudyFrog is used, but today it cannot open them.

## Description
Uses the standard annotation types so other readers display them.

## Acceptance Criteria
- [ ] "Save a copy with annotations" writes highlights and notes into a new PDF that other readers show
- [ ] Title, author and subject can be edited and are written to the copy
- [ ] The sidecar is unchanged and the original is untouched

## Design Notes
Checked by opening the copy with a second tool in the test, where one is available.

## Dependencies
TASK-0114, TASK-0115.

## Open Questions
None.
