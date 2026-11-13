---
id: TASK-0117
type: task
title: "Extract text into Markdown notes with a link back to the page"
status: done
epic: EPIC-022
parents: [STORY-0054]
estimate: 5
labels: [m9, pdf]
acceptance_criteria:
  - "The text of a page, a page range or a selection becomes a Markdown note in a stack the user chooses"
  - "Headings are recovered from the outline or font sizes where possible, and paragraphs are rejoined across line breaks and hyphens"
  - "The note keeps a source link such as `source: documents/book.pdf#page=12`, and the item page shows it as a link that opens the viewer at that page"
  - "A PDF page shows which notes were made from it"
  - "Columns and non-Latin text are handled as far as the library allows, and the limits are listed in docs/specs/pdf.md"
---

## Context
The owner asked for PDF support on 2026-10-31: PDF files are integral to how StudyFrog is used, but today it cannot open them.

## Description
Pure functions from page text blocks to Markdown, tested on generated PDFs.

## Acceptance Criteria
- [x] The text of a page, a page range or a selection becomes a Markdown note in a stack the user chooses
- [x] Headings are recovered from the outline or font sizes where possible, and paragraphs are rejoined across line breaks and hyphens
- [x] The note keeps a source link such as `source: documents/book.pdf#page=12`, and the item page shows it as a link that opens the viewer at that page
- [x] A PDF page shows which notes were made from it
- [x] Columns and non-Latin text are handled as far as the library allows, and the limits are listed in docs/specs/pdf.md

## Design Notes
Uses the existing create use case so the note lands on the board and in the index like any other.

## Dependencies
TASK-0110, TASK-0112.

## Open Questions
None.
