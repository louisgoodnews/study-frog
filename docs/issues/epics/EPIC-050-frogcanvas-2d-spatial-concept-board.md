---
id: EPIC-050
type: epic
title: "FrogCanvas: 2D Spatial Mind-Map and Concept Board"
status: done
parents: []
labels: [m13, graph, ui]
acceptance_criteria:
  - 'Stack directories support optional _canvas.toml spatial layout serialization'
  - 'Spatial quadtree indexing provides viewport culling with 60 FPS pan and zoom'
  - 'Cards, notes, and visual PDF excerpts can be arranged, clustered, and connected'
  - 'Interactive 2D canvas view with node dragging, edge linking, and card previews'
---

## Context
Visual thinkers and learners synthesizing complex domains need spatial layouts to structure concepts beyond linear lists or outliners.

## Description
Deliver FrogCanvas: an infinite 2D spatial canvas connecting cards, notes, and PDF visual crops with force-directed layouts and spatial persistence in _canvas.toml.

## Acceptance Criteria
- [x] Stack directories support optional _canvas.toml spatial layout serialization
- [x] Spatial quadtree indexing provides viewport culling with 60 FPS pan and zoom
- [x] Cards, notes, and visual PDF excerpts can be arranged, clustered, and connected
- [x] Interactive 2D canvas view with node dragging, edge linking, and card previews

## Design Notes
See docs/v0.9.0-roadmap.md.

## Dependencies
None.

## Open Questions
None.
