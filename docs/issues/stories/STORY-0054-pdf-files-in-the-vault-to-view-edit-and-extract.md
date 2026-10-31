---
id: STORY-0054
type: story
title: PDF files in the vault to view, edit and extract
status: backlog
epic: EPIC-022
parents: [EPIC-022]
estimate: 8
labels: [m9, pdf]
acceptance_criteria:
  - "A PDF in the vault is listed, searchable by title and text, and opens in a viewer inside the app"
  - "The viewer shows pages with zoom, page navigation, an outline, text search with highlights and text selection"
  - "A PDF can be annotated (highlights and notes) without changing the original file, and pages can be rotated, deleted, reordered, merged and split into a new file"
  - "Text, selected passages and pictures can be extracted into Markdown notes, cards and questions that keep a link back to the PDF page"
  - "Hostile or broken PDFs cannot crash or hang the app"
---

## Context
The owner asked for PDF support on 2026-10-31: PDF files are integral to how StudyFrog is used, but today it cannot open them.

## Description
Implement viewing, editing and extraction of PDF files, in the tasks below. The estimate is the largest the board allows; the tasks add up to about 55 points, so split the story into viewing, editing and extraction once the library is chosen if that is easier to plan.

## Acceptance Criteria
- [ ] A PDF in the vault is listed, searchable by title and text, and opens in a viewer inside the app
- [ ] The viewer shows pages with zoom, page navigation, an outline, text search with highlights and text selection
- [ ] A PDF can be annotated (highlights and notes) without changing the original file, and pages can be rotated, deleted, reordered, merged and split into a new file
- [ ] Text, selected passages and pictures can be extracted into Markdown notes, cards and questions that keep a link back to the PDF page
- [ ] Hostile or broken PDFs cannot crash or hang the app

## Design Notes
Tasks: TASK-0109 to TASK-0122. Order: the spike and ADR (0109) first; then the crate (0110) and indexing (0111); then viewer (0112) and the safety limits (0113); then annotations, page operations, extraction and the rest, which can overlap. "Edit" means annotating and changing pages, written to a new file or a sidecar, never rewriting the user's original in place and never editing the text inside a PDF (not reliable in any PDF library).

## Dependencies
None.

## Open Questions
1. Does "edit" cover more than annotations and page operations (for example filling forms, redaction, signing)? Default: no.
2. Scanned PDFs have no text. OCR (Tesseract or a language model) is a large dependency; TASK-0118 only detects such pages and says so. Is OCR wanted later?
3. Where should PDFs live: next to the notes in a stack folder (default, so they move with the stack) or in a shared documents folder?
