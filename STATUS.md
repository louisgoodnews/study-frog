# Status

**Phase:** M3 complete; M4 (UI shell) seeded and next. **Date:** 2026-10-30. **Branch:** `main`.

## What exists
- Cargo workspace with 12 empty, documented crates (`crates/studyfrog-*`); `cargo check` passes.
- `docs/`: vision, architecture, principles, glossary, ADRs 0001-0005 (all Accepted),
  roadmap M0-M8 (17 epics, 32 stories, 67 tasks planned), specs, research, contributing guides.
- `docs/issues/`: M0 seeded (EPIC-001/002, STORY-0001..0004, TASK-0001..0011).
- Guardrails in `scripts/`: `check.sh` (fmt, clippy, test, line count, layering, no-OOP),
  `linecount.sh`, `check-deps.sh`, `check-impls.sh`, `check-issues.sh`. Each check was shown to
  fail on a violation. `tools/repo-lint` (standalone, syn + serde-saphyr) backs the last two.
- Config: `rustfmt.toml`, `clippy.toml`, `rust-toolchain.toml` (1.99.1), `.gitignore`.
- Domain code: `studyfrog-core` (data + free functions; the only `impl` blocks are the `Idx<T>` trait
  impls) and `studyfrog-srs` (free functions only).

## Flags and known gaps
- ADR 0004 is accepted but gated on the M6 latency spike (p95 <= 150 ms, else switch to `ort`).
- Benchmark budgets are provisional (rationale documented); real baselines come with the first
  benchmark of each crate.
- Macro-generated impl detection in `repo-lint` is token-level and heuristic.
- Unverified-from-memory citations remain marked `[u]` in `docs/research/` (Ebbinghaus, Bjork,
  Wozniak, FSRS papers, Fitts, Hick, Nielsen, GNOME HIG).
- No git remote is configured; the workspace `repository` field was removed until one exists.
- FSRS-6 pinned in `docs/specs/srs-algorithm.md`; remaining update equations and golden vectors
  are taken from the reference implementation in TASK-0016/0017.

## Flags (please review)
- **ADR 0006 (Proposed):** `studyfrog-service`, the application layer between `ui` and the domain crates.
- **ADR 0007 (Proposed):** settings live in TOML (`studyfrog.toml`, `_stack.toml`), content in Markdown, per your
  suggestion; new pure crate `studyfrog-config`. Reading of "point to Markdown files": config sits beside the
  content and is keyed by folder (and id), not a manifest of paths. Say if you meant a manifest.
- **Decision to confirm:** spaced runs default to `UpdateSchedule`; sequential/shuffled/drill/stalest runs default to
  `PracticeOnly`. Overridable per run.
- Service gaps from M3 are closed: SM-2 per stack, per-stack daily limits, persisted run filters.
- Known limits: the settings renderer creates files and does not preserve comments (the settings UI should use
  `toml_edit`); `studyfrog.toml` is read on demand (no hot reload yet); a day window is supplied by the caller
  (the UI must decide local midnight / rollover); after a scheduler switch `Review.elapsed_days` restarts for the
  new algorithm.
- Lowest-recall queue building is 3.9 ms for 10k items (a `powf` per comparison); cacheable later.

## Next task
**M4:** TASK-0036 (eframe window and navigation; includes the egui accessibility/IME spike) and TASK-0037
(pure Elm-style update loop and effects). Live state: [BOARD.md](BOARD.md).

## M3 summary
- `studyfrog-core`: `RunPlan` (order, filter, effect, seed, limit, requeue) and persisted names.
- `studyfrog-rehearsal`: pure `step` state machine (validate-then-mutate), injected clock, latency without
  paused time, requeue with cap; queue builders for all orders and filters (seeded in-house shuffle).
  Engine step 23 ns; 10k-candidate queues 82 us to 3.9 ms.
- `studyfrog-store`: schema v2, run/event persistence, crash recovery, reviews and FSRS state.
- `studyfrog-service`: end-to-end tests over a real vault and database.

## M2 summary
- `studyfrog-md`: in-house ULID and RFC 3339 codecs, fence/key-block splitter, typed parse and render for
  all four kinds, unknown keys preserved verbatim, `???` splitter, wikilinks, duplicate-id check.
  1000-case round-trip property.
- `studyfrog-store`: SQLite schema v1 + migrations, vault scanner (non-fatal errors, duplicate ids),
  library builder (inbox stack), incremental reindex (rename-safe), polling change detection.
  10k notes: scan+parse 282 ms, cold index 571 ms, warm reindex 43 ms (budget 3 s).
- Decisions: polling instead of a watcher crate; canonical key order with verbatim unknown keys;
  orphans go to an implicit Inbox stack (uid 0).

## M1 summary
- `studyfrog-core`: `Uid`, `Idx<T>`, `Arena<T>`, `Meta`, `Stack`, `Note`, `Card`, `Question`,
  `Library` (one arena per kind), `Rating`, `Review`, `Run`, `RunEvent`, `Latency`, `MemState`,
  `Sm2State`, `UnixMs`.
- `studyfrog-srs`: FSRS-6 (golden test reproduces py-fsrs intervals; 7 properties), SM-2 (hand-computed
  vectors + properties), deterministic fuzz. Benchmarks: FSRS 2.9 ms / 100k cards, SM-2 0.72 ms.
