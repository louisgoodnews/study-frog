---
id: BUG-0009
type: bug
title: "PDF lists show a placeholder title instead of the file name"
status: done
parents: []
estimate: 2
labels: [m11, ui, pdf]
acceptance_criteria:
  - "Every list that shows a PDF names it by its file name (the Home card, the stack page, the search results and the viewer's header)"
  - "The title stored inside the file is added only when it says something: placeholders such as 'PowerPoint-Präsentation', 'Untitled' or the file's own name are left out"
  - "Two files with the same placeholder title can be told apart"
---

## Context
The owner sent a screenshot of the Home page's PDF card on 2026-11-03: two different PDFs are both listed as 'PowerPoint-Präsentation' with 48 and 63 pages, which says too little.

## Description
The PDF index stores the document title, else the file name. Presentation and word processors write a placeholder as the title, so different files look alike.

## Acceptance Criteria
- [x] Every list that shows a PDF names it by its file name (the Home card, the stack page, the search results and the viewer's header)
- [x] The title stored inside the file is added only when it says something: placeholders such as 'PowerPoint-Präsentation', 'Untitled' or the file's own name are left out
- [x] Two files with the same placeholder title can be told apart

## Design Notes
`pdf_names.rs` (pure): `file_name`, `is_placeholder` and `label`. The file name comes from the path the index already holds, so no analysis of the file is needed; the owner offered either the file name or analysing the PDF for its name. A first-page heading could be a later addition. Tests in `tests/pdf_names.rs`.

## Dependencies
None.

## Open Questions
None.
