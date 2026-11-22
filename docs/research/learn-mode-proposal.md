# Learn modes: how a custom run is logged and scheduled (STORY-0198, part 2)

Status: proposal from a three-agent review (data and storage; schedule and learning science; engine and
product), synthesised on 2026-11-22. Read-only research: no code changed by the review. For decision before
building.

## Decision asked
1. What a custom mode's run records.
2. Whether its ratings feed the FSRS/SM-2 schedule like a built-in mode, or stay separate.

## Recommendation in one paragraph
Run a custom mode as a **practice-only, separate-record** mode in v1. Its ratings go to new side tables
(`learn_run`, `learn_step`), not to `review`, `mem_state` or `sm2_state`. Each run stores a snapshot of the
graph it ran, so logs can be replayed. A later comparison with built-in modes uses held-out items and a
delayed test, and only then decides whether to feed the shared schedule (behind an explicit opt-in).

## Where the three reports agree
- **Schedule.** A rating writes a `review` row and updates `mem_state`/`sm2_state` (`rehearse.rs`
  `rate_item_in`). Practice-only runs write none. A custom mode that fed the shared schedule would change
  what built-in queues show, and its ratings would enter the FSRS optimiser, which reads every review
  (`fsrs_optimize.rs`). **Keep custom ratings out of v1.**
- **Storage.** The `run` table's CHECK allows only `flip`, `answer`, `exam` (`schema_exam.rs`), and
  `run_mode_from_name` errors on anything else. An unknown mode makes `run_history` fail as a whole. So the
  custom run must not add a mode value. Use a **sentinel** `run.mode` (`flip` or `answer`, from the graph's
  recall kind) plus a side table that marks the row as custom. This avoids a table rebuild and any change
  to readers of the mode column.
- **Built-in analytics must not see custom runs.** Filter with `AND run_uid NOT IN (SELECT run_uid FROM
  learn_run)` in: `recent_runs` (home), run starts and counts (reports, achievements), `run_history`. A
  test must insert a custom run and check that these ignore it.
- **Engine.** Start, present, recall and rate already exist as engine steps. Branch and log are new, and
  branch is pure interpreter logic. Stop needs one new finish action; `Abort` ends in the wrong state.
- **Keep out of custom runs:** the exam path (`exam_refusal` must stay the first check), pacing
  (`timed_out` ends runs), and the memory mode (its own state machine).

## Proposed schema (v1)
`learn_run` (one row per custom run): `run_uid` (primary key), `mode_id`, `mode_version`, `graph_hash`,
`graph_toml` (the full snapshot the run used), `schedule_effect` (`practice_only` in v1; `update_schedule`
reserved).

`learn_step` (one row per node visit): `run_uid`, `seq`, `item_uid`, `node_id`, `node_kind`, `edge_when`
(the branch condition taken), `rating` (1 to 4), `latency_ms` (paused time excluded), `t_ms`, `outcome`.
Primary key `(run_uid, seq)`.

Both come in one additive migration (`CREATE TABLE` only, no rebuild). Log steps in `learn_step`, not as
new `run_event` kinds: old readers fail on unknown event kinds (`run_load.rs`).

## Engine plan (v1)
- An interpreter in `studyfrog-rehearsal` (`learn_interp.rs`): compile the graph once into an index table;
  hold `node`, `visits` per node, and `last_rating`. It is a pure function `advance(graph, state, event) ->
  drive`, where drive is flip, rate or finish. It wraps the engine; it does not edit `rate()`, which already
  advances the queue, to avoid changing the built-in modes.
- Branch conditions: the four ratings (again, hard, good, easy) and `default`. No score thresholds in v1:
  typed scores arrive asynchronously from the local model and would block the engine.
- **Visit caps are required in the interpreter**, not only in the static check. The engine's requeue does
  not enforce its cap (`requeue`, `step.rs`), so a loop with no limit could run forever.
- The static check (`learn_graph::check`) must also refuse unknown `when` values and a branch with no
  default. It does not do this yet.

## Learning-science basis for the research design (not for v1 code)
- Spacing (Cepeda et al. 2006) and retrieval practice with a delayed test (Roediger and Karpicke 2006) are
  established; Dunlosky et al. 2013 rates both as high utility. These are in `docs/research/learning-science.md`.
- A within-learner comparison needs counterbalanced order, a washout, and items randomised to conditions.
  Items shared across conditions contaminate each other through a shared schedule: the strongest argument
  against shared scheduling for experiments.
- The primary outcome should be delayed recall on held-out items, not reviews per minute, which can be
  gamed with easier items. Ratings are self-report and secondary.
- Not verified, so not to be cited until checked: the exact washout length; the Kornell and Bjork (2008)
  and Rohrer et al. (2015) details; the size of the early-review stability gain in FSRS (`stability.rs` was
  not read in full).

## Editor, minimum for v1
List stored modes; add a node (kind and label); connect two nodes; delete a node or a wire; show the
`reason` lines from the check; save. Canvas drag, zoom and layout can wait (`studyfrog-graph` exists for
later). The editor and the run view can be built in two files of about 200 lines each; split them early.

## Size and line-limit risks
`learn_graph.rs` is at 227 lines and `step.rs` at 223; both must be split before they grow. Move
`Refusal` and `reason` out of `learn_graph.rs` first.

## Acceptance mapping
- Drawn: the editor (above).
- Saved: `learn_mode_store::save` (already built), which refuses invalid graphs.
- Run: interpreter plus the existing run card, which already keeps rating buttons outside the scrolling item area.
- Logged apart: `learn_run` and `learn_step`, with built-in readers filtering them out and an opt-in
  `include_custom` on the custom report.

## Open questions for the owner
1. Accept practice-only v1 with no effect on the schedule?
2. Is a per-run graph snapshot acceptable in size? Runs repeat the TOML; a hash reference is the alternative.
3. Should the custom report be opt-in only (recommended), or shown beside the built-in report?

## Risks, summarised
- A reader that misses the filter counts custom runs as built-in (mitigated by the leakage test).
- An interrupted old build opening a new database: the sentinel mode avoids a CHECK change, so old builds
  see only `flip` or `answer` rows.
- The graph hash must be stable: use a hand-written FNV-1a 64 over the canonical TOML, not `DefaultHasher`,
  which is not stable across Rust releases.
- Loops without a visit cap could run forever (mitigated in the interpreter).
