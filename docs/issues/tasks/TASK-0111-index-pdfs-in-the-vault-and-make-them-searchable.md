---
id: TASK-0111
type: task
title: Index PDFs in the vault and make them searchable
status: backlog
epic: EPIC-022
parents: [STORY-0054]
estimate: 5
labels: [m9, pdf]
acceptance_criteria:
  - "The scanner records every .pdf file (path, size, hash, page count, title) next to the Markdown files"
  - "The text of each page is extracted once, stored in the index and refreshed only when the file's hash changes"
  - "Global search and the Library find a PDF by title and by text, and show the page of the hit"
  - "Large vaults do not slow down: extraction runs off the UI thread and a progress note is shown for a first scan"
  - "A schema migration adds the tables, and the migration test list is updated"
---

## Context
The owner asked for PDF support on 2026-10-31: PDF files are integral to how StudyFrog is used, but today it cannot open them.

## Description
Schema v11 with a document table and a page-text table (full text search if the index already supports it, else a LIKE search with a limit).

## Acceptance Criteria
- [ ] The scanner records every .pdf file (path, size, hash, page count, title) next to the Markdown files
- [ ] The text of each page is extracted once, stored in the index and refreshed only when the file's hash changes
- [ ] Global search and the Library find a PDF by title and by text, and show the page of the hit
- [ ] Large vaults do not slow down: extraction runs off the UI thread and a progress note is shown for a first scan
- [ ] A schema migration adds the tables, and the migration test list is updated

## Design Notes
The backup already carries every vault file; check that a large PDF does not break it and mention the size in the summary.

## Dependencies
TASK-0110.

## Open Questions
None.
