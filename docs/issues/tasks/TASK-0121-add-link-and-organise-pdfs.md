---
id: TASK-0121
type: task
title: Add, link and organise PDFs
status: backlog
epic: EPIC-022
parents: [STORY-0054]
estimate: 3
labels: [m9, pdf]
acceptance_criteria:
  - "'Add PDF…' in the Create menu and the Library copies a file into the chosen stack folder"
  - "The Library shows PDFs in their stack with a pdf label and the page count; they can be renamed and moved with the stack"
  - "A Markdown link or image-style reference to a PDF with `#page=N` opens the viewer at that page"
  - "The home page's recent activity and the board treat PDFs like other items where it makes sense"
---

## Context
The owner asked for PDF support on 2026-10-31: PDF files are integral to how StudyFrog is used, but today it cannot open them.

## Description
Copy rather than link, so the vault stays self-contained and backups complete.

## Acceptance Criteria
- [ ] "Add PDF…" in the Create menu and the Library copies a file into the chosen stack folder
- [ ] The Library shows PDFs in their stack with a pdf label and the page count; they can be renamed and moved with the stack
- [ ] A Markdown link or image-style reference to a PDF with `#page=N` opens the viewer at that page
- [ ] The home page's recent activity and the board treat PDFs like other items where it makes sense

## Design Notes
No move or delete of PDFs outside the vault.

## Dependencies
TASK-0111, TASK-0112.

## Open Questions
None.
