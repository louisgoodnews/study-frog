---
id: TASK-0113
type: task
title: Safety limits for hostile or broken PDFs
status: in_progress
epic: EPIC-022
parents: [STORY-0054]
estimate: 3
labels: [m9, pdf]
acceptance_criteria:
  - "Size, page count, object count and render time have limits, and a PDF over a limit is refused with a sentence that names the limit"
  - "A panic or a hang in the PDF library cannot take the window down: the work runs on the worker thread under a timeout"
  - "PDF JavaScript and launch actions never run, and links to the web open only after a click and a confirmation"
  - "A small corpus of malformed files (truncated, looping page tree, huge image, bad xref) is tested"
---

## Context
The owner asked for PDF support on 2026-10-31: PDF files are integral to how StudyFrog is used, but today it cannot open them.

## Description
Limits in the crate, isolation in the executor.

## Acceptance Criteria
- [ ] Size, page count, object count and render time have limits, and a PDF over a limit is refused with a sentence that names the limit
- [ ] A panic or a hang in the PDF library cannot take the window down: the work runs on the worker thread under a timeout
- [ ] PDF JavaScript and launch actions never run, and links to the web open only after a click and a confirmation
- [ ] A small corpus of malformed files (truncated, looping page tree, huge image, bad xref) is tested

## Design Notes
Never write the original file; opening is read-only.

## Dependencies
TASK-0110.

## Open Questions
None.
