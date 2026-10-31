---
id: TASK-0110
type: task
title: The studyfrog-pdf crate with documents, pages, text and pictures as data
status: done
epic: EPIC-022
parents: [STORY-0054]
estimate: 5
labels: [m9, pdf]
acceptance_criteria:
  - "A new crate with free functions: open bytes, page count and sizes, title and metadata, outline, text per page with positions (lines and words), pictures per page, render a page to pixels at a scale"
  - "The chosen library is used only inside the crate; the rest of the workspace sees plain data"
  - "The layering script knows the crate and nothing depends on it except the service layer and the UI"
  - "Tests use small PDFs generated in the tests (text, two columns, an image, an outline, a broken file)"
---

## Context
The owner asked for PDF support on 2026-10-31: PDF files are integral to how StudyFrog is used, but today it cannot open them.

## Description
Pure data in and out; rendering returns an RGBA buffer the UI turns into a texture.

## Acceptance Criteria
- [x] A new crate with free functions: open bytes, page count and sizes, title and metadata, outline, text per page with positions (lines and words), pictures per page, render a page to pixels at a scale
- [x] The chosen library is used only inside the crate; the rest of the workspace sees plain data
- [x] The layering script knows the crate and nothing depends on it except the service layer and the UI
- [x] Tests use small PDFs generated in the tests (text, two columns, an image, an outline, a broken file)

## Design Notes
Honour the repo rules: no inherent impl, files under 250 lines, nesting under 3.

## Dependencies
TASK-0109.

## Open Questions
None.
