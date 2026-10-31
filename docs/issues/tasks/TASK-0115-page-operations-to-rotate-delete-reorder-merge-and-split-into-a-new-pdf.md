---
id: TASK-0115
type: task
title: Page operations to rotate, delete, reorder, merge and split into a new PDF
status: done
epic: EPIC-022
parents: [STORY-0054]
estimate: 5
labels: [m9, pdf]
acceptance_criteria:
  - "Pages can be rotated, deleted and reordered in a thumbnail view, with undo until the result is saved"
  - "Several PDFs can be merged, and a PDF can be split by page ranges"
  - "The result is always written as a new file the user names; the original is never changed"
  - "Highlights and notes follow the pages that move; the outline is not carried into the new file, and the limits are written in the docs (TASK-0122)"
---

## Context
The owner asked for PDF support on 2026-10-31: PDF files are integral to how StudyFrog is used, but today it cannot open them.

## Description
This task is the whole of the owner's "edit" apart from annotations (TASK-0114): rotate, delete, reorder, merge, split. An editing library (lopdf or what the ADR chooses) behind the studyfrog-pdf crate.

## Acceptance Criteria
- [x] Pages can be rotated, deleted and reordered in a thumbnail view, with undo until the result is saved
- [x] Several PDFs can be merged, and a PDF can be split by page ranges
- [x] The result is always written as a new file the user names; the original is never changed
- [x] Highlights and notes follow the pages that move; the outline is not carried into the new file, and the limits are written in the docs (TASK-0122)

## Design Notes
Keep the operations as pure functions from a document and a plan to bytes, so they can be tested with generated PDFs.

## Dependencies
TASK-0109, TASK-0110, TASK-0112.

## Open Questions
None.
