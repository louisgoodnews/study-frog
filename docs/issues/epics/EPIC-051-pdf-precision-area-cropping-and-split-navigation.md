---
id: EPIC-051
type: epic
title: "PDF Precision Area Cropping and 1-Click Split Navigation"
status: backlog
parents: []
labels: [m13, pdf, ui]
acceptance_criteria:
  - 'Lasso rectangular selection tool crops visual PDF regions directly into cards'
  - 'Coordinate frontmatter anchor format tracks source document, page, and bounding box'
  - '1-click split navigation button opens PDF viewer beside rehearsal card flashing the excerpt'
  - 'LRU memory and disk rasterization cache maintains 60 FPS rehearsal pacing'
---

## Context
Students studying histology, engineering, anatomy, and mathematics rely heavily on textbook diagrams that lose context when converted to pure text.

## Description
Enable visual rectangular area cropping from PDFs into cards with persistent source coordinate anchors and instant 1-click split-screen document jumping.

## Acceptance Criteria
- [ ] Lasso rectangular selection tool crops visual PDF regions directly into cards
- [ ] Coordinate frontmatter anchor format tracks source document, page, and bounding box
- [ ] 1-click split navigation button opens PDF viewer beside rehearsal card flashing the excerpt
- [ ] LRU memory and disk rasterization cache maintains 60 FPS rehearsal pacing

## Design Notes
See docs/v0.9.0-roadmap.md.

## Dependencies
None.

## Open Questions
None.
