---
id: STORY-0153
type: story
title: "FrogCanvas Data Model and Spatial Serialization"
status: backlog
epic: EPIC-050
parents: ['EPIC-050']
estimate: 5
labels: [m13, graph]
acceptance_criteria:
  - 'Data structs represent CanvasNode, CanvasEdge, CanvasFrame, and CanvasDoc'
  - 'Serialization and deserialization to _canvas.toml in stack directories'
  - 'Spatial quadtree index efficiently queries nodes and edges within viewport bounds'
---

## Context
Core data model for FrogCanvas spatial concept maps.

## Description
Implement pure data representations of spatial canvas nodes, edges, and frames, with TOML serialization and quadtree viewport culling.

## Acceptance Criteria
- [ ] Data structs represent CanvasNode, CanvasEdge, CanvasFrame, and CanvasDoc
- [ ] Serialization and deserialization to _canvas.toml in stack directories
- [ ] Spatial quadtree index efficiently queries nodes and edges within viewport bounds

## Design Notes
See docs/v0.9.0-roadmap.md.

## Dependencies
EPIC-050

## Open Questions
None.
