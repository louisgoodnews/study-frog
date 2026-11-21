---
id: STORY-0198
type: story
title: "Visual node-based editor for learn modes"
status: backlog
epic: EPIC-066
parents: ['EPIC-066']
estimate: 8
labels: [m14, research, ui]
acceptance_criteria:
  - 'a learn mode can be drawn as a graph of nodes and edges in the app, saved in the vault and run'
  - 'a graph that is not valid (no start, a loop without a stop, an unknown node) is refused with the reason before it can run'
  - 'a run of a custom mode logs apart from the built-in modes, so the two can be compared'
---

## Context
A learn mode is a sequence of presentation, recall and rating decisions. A researcher wants to try a new one, such as interleaving two decks with a different rating rule, and compare it with the built-in modes.

## Description
An editor canvas with a palette of node types (present, recall, rate, schedule, log, branch), wires between them, and a check that the graph is valid. A saved graph runs through the rehearsal engine; each node is a pure step the engine can test. Store the graph as data in the vault, not in code. The first version needs linear and branching graphs; a loop needs a stop rule.

## Acceptance Criteria
- [ ] a learn mode can be drawn as a graph of nodes and edges in the app, saved in the vault and run
- [ ] a graph that is not valid (no start, a loop without a stop, an unknown node) is refused with the reason before it can run
- [ ] a run of a custom mode logs apart from the built-in modes, so the two can be compared

## Design Notes
Which node types are needed first?

## Dependencies
EPIC-066, and the features named above.

## Open Questions
None.
