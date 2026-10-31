---
id: TASK-0114
type: task
title: Annotations as highlights and notes in a sidecar file
status: backlog
epic: EPIC-022
parents: [STORY-0054]
estimate: 5
labels: [m9, pdf]
acceptance_criteria:
  - "The viewer can highlight a text selection and add a note to a point or a selection, in a few colours"
  - "Annotations are stored next to the PDF in a sidecar file in plain text, never inside the original, and survive a rename of the vault folder"
  - "An annotation remembers the page and the text it covers, so it can be found again if the PDF is replaced by a new revision of the same file"
  - "A list of the document's annotations lets the user jump to each, edit and delete it"
  - "Annotations are included in backups and in the search"
---

## Context
The owner asked for PDF support on 2026-10-31: PDF files are integral to how StudyFrog is used, but today it cannot open them.

## Description
A small TOML sidecar `name.pdf.notes.toml` (ADR 0007 style) with page, rectangles, text and colour.

## Acceptance Criteria
- [ ] The viewer can highlight a text selection and add a note to a point or a selection, in a few colours
- [ ] Annotations are stored next to the PDF in a sidecar file in plain text, never inside the original, and survive a rename of the vault folder
- [ ] An annotation remembers the page and the text it covers, so it can be found again if the PDF is replaced by a new revision of the same file
- [ ] A list of the document's annotations lets the user jump to each, edit and delete it
- [ ] Annotations are included in backups and in the search

## Design Notes
Writing the annotations into a copy of the PDF is TASK-0116.

## Dependencies
TASK-0112.

## Open Questions
None.
