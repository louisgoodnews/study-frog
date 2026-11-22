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

## Implementation (part 2, backend)
- Decisions taken (owner, 2026-11-22): practice-only v1 with no effect on the schedule; a full graph snapshot per run; the custom report opt-in.
- Migration 31 (`schema_learn.rs`): `learn_run` (mode id, version, FNV-1a hash, the snapshot, `practice_only`) and `learn_step` (path per item). Additive tables; the `run` row is a `flip` run, so no CHECK changes.
- `studyfrog-rehearsal` `learn_interp::walk`: from a node, one rating at a time, follows the wire (branch by rating, else `default`), stops at a rate node, a stop, or a loop cap (3 visits per node). `learn_refusal` holds the refusal sentences; the check also refuses unknown conditions and a branch with no default.
- `studyfrog-service` `learn_run::start` runs a checked stored mode practice-only and keeps the snapshot; `record_rating` (called inside `record_step`'s transaction) walks the graph and stores the steps. A test shows the walk and that no `review` row is written.
- Built-in readers (recent runs, run starts, the finished count) leave custom runs out (`store::learn::BUILTIN_RUNS`), tested.
- Changed behaviour, stated: in v1 the graph decides the recorded path and the branches; the run's queue order is still the built-in engine's. A custom run therefore records what the graph would do with each rating.

## Implementation (part 2, form editor)
- A form-based editor (not a canvas): "Learn modes" on the Rehearse setup opens a dialog that lists the stored modes, opens one, names it, adds and removes nodes (kind and label; ids are generated), adds and removes wires (from, to, condition or plain), shows the reasons it cannot be saved, and saves. Save is refused while there are reasons; the check runs after every edit.
- Pure state and update in `learn_editor.rs` (tests in `tests/learn_editor.rs`); the file work is effects in `exec_learn.rs`; the form in `view/learn_editor.rs`; a screenshot demo `--screenshot learn-editor`.

## Still open (part 2)
- The canvas (drawn nodes and wires) is not built; the form editor above is the first version.
- The opt-in custom report screen (the store function `custom_mode_totals` exists).
- Acceptance criterion 1 ("drawn in the app") is met by the form editor only, not as a canvas.
- Starting a custom run from the app: done. "Start a run" in the editor runs the saved copy of the open mode (`exec_learn` calls `learn_run::start`, the run view takes over). Tested in `tests/learn_run_app.rs` and `tests/learn_editor.rs`; not yet tried by hand in a window.

## Dependencies
EPIC-066, and the features named above.

## Open Questions
None.
