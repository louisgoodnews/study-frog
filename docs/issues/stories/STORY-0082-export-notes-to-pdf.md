---
id: STORY-0082
type: story
title: Export notes to PDF
status: backlog
epic: EPIC-026
parents: [EPIC-026]
estimate: 8
labels: [m11, pdf, md, ui]
acceptance_criteria:
  - "A note, card or question, several chosen items, or a whole stack can be exported to one PDF file from the application, with the Markdown rendered as it is in the preview (headings, lists, tables, code, pictures, links, colour and underline)"
---

## Context
The owner asked on 2026-11-04 for a new feature: notes can be exported to PDF, which needs a converter from Markdown to PDF.

## Description
Let the user hand a note, a selection or a stack to someone else, or print it, as a PDF made by StudyFrog itself.

## Acceptance Criteria
- [ ] A note, card or question, several chosen items, or a whole stack can be exported to one PDF file from the application, with the Markdown rendered as it is in the preview (headings, lists, tables, code, pictures, links, colour and underline)

## Design Notes
Tasks TASK-0228 to TASK-0232. StudyFrog can read, draw and edit PDFs (ADR 0010) but cannot yet write a new document from text; that writer and the Markdown layout are the new part.

## Dependencies
None.

## Open Questions
None.
