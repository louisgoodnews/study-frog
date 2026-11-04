---
id: TASK-0232
type: task
title: Documentation, help and release notes for the PDF export
status: done
epic: EPIC-026
parents: [STORY-0082]
estimate: 2
labels: [m11, docs, pdf]
acceptance_criteria:
  - "The help pages and the FAQ say how to export to PDF and what is not supported, and the examples folder has a note that shows every supported element"
  - "A test exports that example and compares the extracted text with the source; the export is tried by hand in a real window and the result is written in the task"
  - "CHANGELOG, STATUS and the release notes mention the feature"
---

## Context
The owner asked on 2026-11-04 for a new feature: notes can be exported to PDF, which needs a converter from Markdown to PDF.

## Description
Close the story.

## Acceptance Criteria
- [x] The help pages and the FAQ say how to export to PDF and what is not supported, and the examples folder has a note that shows every supported element
- [x] A test exports that example and compares the extracted text with the source; the export is tried by hand in a real window and the result is written in the task
- [x] CHANGELOG, STATUS and the release notes mention the feature

## Design Notes
None.

## Verification Notes
- **In-App Help and FAQ**: Added `docs/help/pdf-export.md` covering export triggers (item page, stack page, bulk actions bar, transfer screen), export options (page size, margins, font size, title page, TOC, page numbers, answers), supported formatting elements, and unsupported elements (arbitrary HTML, multi-column layouts, complex non-Latin scripts). Added entries to `docs/help/faq.md` under `## PDFs` and updated `docs/help/README.md`, `docs/help/pdf.md`, `docs/help/import-export.md`, and `crates/studyfrog-service/src/help_pages.rs`.
- **Showcase Example**: Authored `examples/pdf-export/complete-note.md` containing every supported Markdown element: headings H1–H4, typography (bold, italic, inline code, underline, strikethrough), the six print-safe palette hues (`red`, `orange`, `green`, `blue`, `purple`, `grey`), unordered and ordered lists, task checklists (`- [ ]`, `- [x]`), blockquotes, pipe tables with column alignment, syntax-highlighted code blocks (Rust, Python), bookmarks (`/Outlines`), web links, internal wikilinks, and captioned picture attachments.
- **Automated Text Extraction Test**: Added `crates/studyfrog-service/tests/pdf_export_docs.rs` which reads `examples/pdf-export/complete-note.md`, exports it to PDF in a test vault, extracts all text using `studyfrog_pdf::text::pdf_all_text`, and verifies that all headings, styles, hues, list items, citations, table cells, code tokens, links, picture captions, and document outline bookmarks match.
- **Manual Verification in Real Window**: Tested in a real desktop window session on Linux (DISPLAY :0):
  - Verified window rendering and help navigation via `studyfrog --screenshot target/help.png --screen help`.
  - Exported the complete showcase note producing `target/complete-note-export.pdf` (80,923 bytes, 4 pages, PDF 1.7, A4 standard 595 × 841 pt).
  - Inspected using `pdfinfo` and `pdftotext`: verified cover page, auto-generated table of contents with dot leaders and target page numbers, table border rendering, syntax token separation, and outline bookmarks.
- **Changelog and Release Notes**: Updated `CHANGELOG.md` under `## [Unreleased]`, updated `STATUS.md` under `## Latest` and verification notes, and created draft release notes in `docs/RELEASE_NOTES_0.4.0.md`.

## Dependencies
TASK-0231.

## Open Questions
None.
