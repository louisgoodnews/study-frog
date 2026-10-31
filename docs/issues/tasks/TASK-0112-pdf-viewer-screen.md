---
id: TASK-0112
type: task
title: PDF viewer screen
status: done
epic: EPIC-022
parents: [STORY-0054]
estimate: 8
labels: [m9, pdf]
acceptance_criteria:
  - "A viewer opens from the search and shows one page at a time (the Library, links and the item page open it in TASK-0117 and TASK-0121)"
  - "Zoom (fit width, fit page, percent), next and previous page, jump to page, thumbnails, outline"
  - "Text search with every hit highlighted and a way through the hits"
  - "Text can be selected and copied"
  - "Pages are rendered off the UI thread and cached, and the window never waits for a render"
  - "Every control is named for screen readers and the extracted text of the page is available to them"
---

## Context
The owner asked for PDF support on 2026-10-31: PDF files are integral to how StudyFrog is used, but today it cannot open them.

## Description
A new screen and sidebar entry; the render cache is an LRU keyed by file hash, page and scale.

## Acceptance Criteria
- [x] A viewer opens from the search and shows one page at a time (the Library, links and the item page open it in TASK-0117 and TASK-0121)
- [x] Zoom (fit width, fit page, percent), next and previous page, jump to page, thumbnails, outline
- [x] Text search with every hit highlighted and a way through the hits
- [x] Text can be selected and copied
- [x] Pages are rendered off the UI thread and cached, and the window never waits for a render
- [x] Every control is named for screen readers and the extracted text of the page is available to them

## Design Notes
Measure: first page under 200 ms on a typical 50-page PDF, scrolling stays at the frame rate. Record the numbers in the benchmarking doc.

## Dependencies
TASK-0110, TASK-0111, TASK-0113.

## Open Questions
None.
