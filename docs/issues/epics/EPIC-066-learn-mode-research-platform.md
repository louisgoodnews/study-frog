---
id: EPIC-066
type: epic
title: "Learn-mode research platform: visual node-based technique editor"
status: done
parents: []
labels: [m14, research, ui]
acceptance_criteria:
  - 'A learning or rehearsal technique can be drawn as a graph in the app, without writing Rust'
  - 'A technique built this way runs through the rehearsal engine and is logged apart from the built-in ones, so it can be compared'
---

## Context
StudyFrog is built for scientific research. Each learning or rehearsal technique is fixed in code, so an experiment with a new one needs a code change and a release.

## Description
A visual, node-based editor in which a learn mode is a graph: nodes for presenting, recalling, rating, scheduling and logging, joined by edges. The graph is saved in the vault and run by the same engine as the built-in modes.

## Acceptance Criteria
- [x] A learning or rehearsal technique can be drawn as a graph in the app, without writing Rust
- [x] A technique built this way runs through the rehearsal engine and is logged apart from the built-in ones, so it can be compared

## Design Notes
Keep the engine pure and the graph a data format; the editor is a view over it.

## Dependencies
The rehearsal engine (studyfrog-rehearsal) and the control protocol (EPIC-064).

## Open Questions
None.
