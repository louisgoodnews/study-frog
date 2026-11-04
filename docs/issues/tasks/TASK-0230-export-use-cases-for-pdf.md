---
id: TASK-0230
type: task
title: Export use cases for PDF in the service
status: backlog
epic: EPIC-026
parents: [STORY-0082]
estimate: 5
labels: [m11, pdf, store]
acceptance_criteria:
  - "One item, a chosen set of items, or a stack (with its sub-stacks and the items it references, each once) can be exported to a PDF file chosen by the user, outside the vault or inside it"
  - "Pictures named by the Markdown are read from the vault, a missing picture is shown as a labelled gap and reported, and wikilinks become the linked item's title (a link to a chosen item in the same file)"
  - "The options are page size, margins, font size, whether to add a title page, a table of contents, page numbers and the answers of cards and questions"
  - "Hostile paths and names are refused; a failed export leaves no half-written file; a large export reports progress and can be cancelled"
---

## Context
The owner asked on 2026-11-04 for a new feature: notes can be exported to PDF, which needs a converter from Markdown to PDF.

## Description
The use cases between the vault and the layout.

## Acceptance Criteria
- [ ] One item, a chosen set of items, or a stack (with its sub-stacks and the items it references, each once) can be exported to a PDF file chosen by the user, outside the vault or inside it
- [ ] Pictures named by the Markdown are read from the vault, a missing picture is shown as a labelled gap and reported, and wikilinks become the linked item's title (a link to a chosen item in the same file)
- [ ] The options are page size, margins, font size, whether to add a title page, a table of contents, page numbers and the answers of cards and questions
- [ ] Hostile paths and names are refused; a failed export leaves no half-written file; a large export reports progress and can be cancelled

## Design Notes
A pure plan lists the items in order; the executor reads the files and calls the layout of TASK-0229. Cards and questions show the front and, as chosen, the back. Progress and cancel follow the existing worker effects.

## Dependencies
TASK-0229.

## Open Questions
None.
