---
id: STORY-0155
type: story
title: "PDF Visual Bounding Box Cropping"
status: backlog
epic: EPIC-051
parents: ['EPIC-051']
estimate: 5
labels: [m13, pdf]
acceptance_criteria:
  - 'Lasso selection tool in PDF viewer enables dragging rectangular bounding boxes'
  - 'Frontmatter coordinate format source: path#page=N&bbox=x,y,w,h is parsed and rendered'
  - 'Cropped excerpts are rasterized and cached in LRU memory and disk storage'
---

## Context
Visual capture from PDF textbooks into flashcards.

## Description
Implement rectangular lasso cropping in the PDF reader, frontmatter coordinate metadata persistence, and excerpt rasterization caching.

## Acceptance Criteria
- [ ] Lasso selection tool in PDF viewer enables dragging rectangular bounding boxes
- [ ] Frontmatter coordinate format source: path#page=N&bbox=x,y,w,h is parsed and rendered
- [ ] Cropped excerpts are rasterized and cached in LRU memory and disk storage

## Design Notes
See docs/v0.9.0-roadmap.md.

## Dependencies
EPIC-051

## Open Questions
None.
