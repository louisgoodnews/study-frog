---
id: STORY-0198
type: story
title: "Visual node-based editor for learn modes"
status: in_progress
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
Node types for the first version (decided): start, present, recall, rate, schedule, log, branch, stop. A branch needs at least two ways on, each with a condition; a loop needs a stop node or a branch that can leave it.

## Implementation (part 1)
- `studyfrog-rehearsal` `learn_graph`: the graph model and `check`, which returns every reason a graph cannot run (no or several starts, duplicate ids, wires to unknown nodes, unreachable nodes, dead ends, a stop with a way on, a branch with fewer than two ways, a loop without a stop). `reason` gives the sentence. Tests in `tests/learn_graph.rs`.
- `studyfrog-service` `learn_mode_store`: TOML in `<vault>/learn_modes/<name>.toml`; `save` refuses a graph with reasons and writes nothing; `load`, `list`, `to_toml`, `from_toml`. Tests in `tests/learn_mode_store.rs`.
- Not yet: the canvas editor (draw, wire, select, delete); running a custom mode through the rehearsal engine (the criteria's "run"); the separate log of a custom run, so the two can be compared. These stay open, so the story is in progress.

## Dependencies
EPIC-066, and the features named above.

## Open Questions
None.
