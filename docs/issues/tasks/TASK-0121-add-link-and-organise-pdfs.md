---
id: TASK-0121
type: task
title: "Add, link and organise PDFs"
status: done
epic: EPIC-022
parents: [STORY-0054]
estimate: 3
labels: [m9, pdf]
acceptance_criteria:
  - "'Add PDF…' in the Create menu and the Library copies a file into the chosen stack folder"
  - "The Library shows PDFs in their stack with a pdf label and the page count; they can be renamed and moved with the stack"
  - "A Markdown link or image-style reference to a PDF with `#page=N` opens the viewer at that page (an image-style reference is shown as a link)"
  - "The home page lists the PDFs that changed most recently; the board needs no change"
---

## Context
The owner asked for PDF support on 2026-10-31: PDF files are integral to how StudyFrog is used, but today it cannot open them.

## Description
PDFs live inside the vault folder (owner's decision, 2026-10-31), so the vault stays self-contained and backups complete. The scanner indexes every .pdf under the vault, so PDFs the user already keeps there are found without being moved; Add PDF copies a file into the chosen stack folder.

## Acceptance Criteria
- [x] "Add PDF…" in the Create menu and the Library copies a file into the chosen stack folder
- [x] The Library shows PDFs in their stack with a pdf label and the page count; they can be renamed and moved with the stack
- [x] A Markdown link or image-style reference to a PDF with `#page=N` opens the viewer at that page (an image-style reference is shown as a link)
- [x] The home page lists the PDFs that changed most recently; the board needs no change

## Design Notes
No move or delete of PDFs outside the vault.

## Dependencies
TASK-0111, TASK-0112.

## Open Questions
None.
