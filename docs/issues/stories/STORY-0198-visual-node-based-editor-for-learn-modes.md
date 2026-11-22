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

## Implementation (part 3, the queue)
- `studyfrog-rehearsal` `learn_path`: a custom run's graph decides whether a rated item comes back in the run. Each item keeps its place (the rate node it waits at) and its node counts; a rating walks the graph (`learn_interp::walk`), and the item is shown again only when the walk ends at a rate node. A stop, a loop at its cap, or no way on takes it out of the run.
- The session carries the path (`Session.learn`); the engine's `rate` asks it when there is one, and keeps the built-in rule (Again, if the plan requeues) otherwise.
- The service stores the same decision: `learn_run::record_rating` returns it, and `record_step` requeues the stored queue on it (for built-in runs, on Again as before).
- The custom start hands the graph to `begin_run`, so the live session gets the path.
- Tests: `tests/learn_queue.rs` (engine: Again loops back, Good out, built-in unchanged), `learn_run_service` (the stored decision), `learn_run_app` (the start carries the graph).
- Not yet: a resumed run (the app does not resume runs from disk, so the live path is always complete); a custom run's queue order (the built-in order still picks the next item; only the requeue follows the graph).

## Implementation (part 4, the report)
- The custom report is opt-in per mode: "Show report" in the editor reads the runs of the saved mode (`studyfrog-store` `learn_report::mode_report`): runs and finished runs, items reached, entries per step, ratings given, branches taken. Nothing is read until the button is pressed.
- Built-in reports are unchanged: their queries already exclude custom runs (`BUILTIN_RUNS`). Tested in `tests/learn_report_store.rs` (a built-in run is not counted under the mode) and `tests/learn_editor.rs` (the request is made only for a saved mode).

## Implementation (part 5, the queue order)
- A custom run's returning item goes back into the queue after a number of other items, by its rating: Again after 2, Hard after 4, Good after 6, Easy after 8 (`learn_path::gap`). These are the first version's defaults, chosen here, not specified by the story; they are a single function to change.
- Built-in runs keep their order: an Again goes to the end of the queue, as before.
- The live queue is authoritative. The stored `run_queue` rows still record a returning item at the end, not at its place: nothing reads them for a run in progress (runs are not resumed from disk), so they are a known difference, to fix if resume is added.
- Tests: `tests/learn_queue.rs` (an Again item comes back after two others: 1,2,3,1,4,5), `learn_run_service` (the stored placement is the same rule).

## Implementation (part 6, the canvas)
- The learn mode is drawn on a canvas in the editor: each step is a box at its place, each wire an arrow (a wire back to a step on the left is an arc above the boxes), a condition written on its wire. Click empty space to add a step of the chosen kind there; drag a step to move it; click a step to select it; "Delete selected" removes it with its wires. "Connect" turns clicks into wires: click one step, then another; the wire takes the condition chosen in the form.
- Places are saved with the mode in a `[layout]` table of the mode file (presentation only: the graph does not depend on it); a mode saved without one lays out on a grid.
- The form below the canvas is still there and shows the same graph; the form remains the way to pick a condition, remove a wire, or see each step.
- Not yet: selecting or deleting a wire on the canvas (use the form), drawing a wire from a step to itself (it is not drawn), and a hand test of dragging in a real window (the canvas was checked in screenshots and by its message tests).

## Still open (part 2)
- The canvas (drawn nodes and wires) is not built; the form editor above is the first version.
- The opt-in custom report screen (the store function `custom_mode_totals` exists).
- Acceptance criterion 1 ("drawn in the app") is met by the form editor only, not as a canvas.
- Starting a custom run from the app: done. "Start a run" in the editor runs the saved copy of the open mode (`exec_learn` calls `learn_run::start`, the run view takes over). Tested in `tests/learn_run_app.rs` and `tests/learn_editor.rs`; not yet tried by hand in a window.

## Dependencies
EPIC-066, and the features named above.

## Open Questions
None.
