---
id: TASK-0228
type: task
title: Specify the Markdown to PDF export and choose the writer (ADR)
status: backlog
epic: EPIC-026
parents: [STORY-0082]
estimate: 3
labels: [m11, pdf, docs, decision]
acceptance_criteria:
  - "docs/specs/pdf-export.md says which Markdown the converter supports (the syntax of TASK-0197, TASK-0214, TASK-0215 and TASK-0216), how each element looks on the page, the page sizes and margins, headers, footers and page numbers, how wikilinks, pictures, the table of contents and text macros come out, and what is left out"
  - "An ADR compares building the layout and the PDF writer in-house with a crate (for example printpdf, genpdf or typst-as-a-library), by output quality, fonts and non-Latin text, size, licence and speed, and records the choice with the build-or-buy note of rule P10"
  - "The owner has accepted the spec and the ADR"
---

## Context
The owner asked on 2026-11-04 for a new feature: notes can be exported to PDF, which needs a converter from Markdown to PDF.

## Description
Write the export down before building it.

## Acceptance Criteria
- [ ] docs/specs/pdf-export.md says which Markdown the converter supports (the syntax of TASK-0197, TASK-0214, TASK-0215 and TASK-0216), how each element looks on the page, the page sizes and margins, headers, footers and page numbers, how wikilinks, pictures, the table of contents and text macros come out, and what is left out
- [ ] An ADR compares building the layout and the PDF writer in-house with a crate (for example printpdf, genpdf or typst-as-a-library), by output quality, fonts and non-Latin text, size, licence and speed, and records the choice with the build-or-buy note of rule P10
- [ ] The owner has accepted the spec and the ADR

## Design Notes
Questions to settle: fonts (a bundled font under a free licence, or the system's), right-to-left and CJK text, whether the PDF is tagged for accessibility, selectable and searchable text (it must be), one file per item or one for a selection, and the look of a stack export (cover page, contents, one item per page or flowing).

## Dependencies
None.

## Open Questions
None.
