---
id: TASK-0228
type: task
title: "Specify the Markdown to PDF export and choose the writer (ADR)"
status: done
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
- [x] docs/specs/pdf-export.md says which Markdown the converter supports (the syntax of TASK-0197, TASK-0214, TASK-0215 and TASK-0216), how each element looks on the page, the page sizes and margins, headers, footers and page numbers, how wikilinks, pictures, the table of contents and text macros come out, and what is left out
- [x] An ADR compares building the layout and the PDF writer in-house with a crate (for example printpdf, genpdf or typst-as-a-library), by output quality, fonts and non-Latin text, size, licence and speed, and records the choice with the build-or-buy note of rule P10
- [x] The owner has accepted the spec and the ADR

## Design Notes
Written as [ADR 0012](../../adr/0012-markdown-to-pdf-writer.md) and the specification in [docs/specs/pdf-export.md](../../specs/pdf-export.md). Option A (in-house layout engine and PDF writer on top of lopdf) chosen per Rule P10; the owner's decisions of 2026-11-04.

Settled design decisions:
- Fonts: Free open-source bundled fonts (Inter / Liberation Sans and JetBrains Mono / Liberation Mono) under OFL / Apache 2.0 with TrueType cmap and ToUnicode CMaps for searchable and selectable text.
- Character handling: Non-Latin and CJK missing glyphs render a safe replacement box (`□`) rather than panicking; full CJK font bundling deferred to avoid 40+ MB font bloat.
- Accessibility: Tagged PDF structure tree (`/StructTreeRoot`), document info metadata (`/Info`), and outline tree (`/Outlines`) built from headings for screen readers and navigation.
- Page geometry: A4 (default metric) and US Letter (default US), 54 pt (0.75 in) margins, running headers with title and footers with page numbers (suppressed on cover pages).
- Export modes: Single items start immediately with heading on page 1 without cover sheet; stack exports feature a cover page, full table of contents with page links, and internal wikilinks mapped to internal `/GoTo` links.
- Sanitization: All scripts, hostile tags, and dangerous link actions (`/Launch`, `/JavaScript`, `/File`) are stripped; only whitelisted HTML spans (`<u>`, `<span style="color:#...">`) and safe links (`/URI`, `/GoTo`) are emitted. Path traversal attacks outside `assets/` are rejected.

## Dependencies
None.

## Open Questions
None.
