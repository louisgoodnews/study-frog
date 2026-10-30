# Changelog

All notable changes to StudyFrog are recorded here, newest first.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). The project is pre-release, so
everything is under `Unreleased` until the first tagged version. Each entry names the work item
(`TASK-xxxx`) it belongs to where one exists.

## [Unreleased]

### Added
- Link graph screen: the wikilink graph as a force-directed picture with pan, zoom, hover, click-to-open, a
  re-run button, an accessible item list, and live refresh that keeps node positions. New pure crate
  `studyfrog-graph` (SoA layout, uniform-grid repulsion, deterministic, zero allocation per step): 5k nodes /
  10k edges step in 4.0 ms; verified allocation-free with dhat. Items now show when they are next due
  (TASK-0042, TASK-0043). EPIC-009 and phase M4 complete.
- Editor: open any item from the browser, edit title, tags and text areas with a live Markdown preview (headings,
  emphasis, lists, task items, quotes, code, wikilinks and URL links), undo/redo, save with conflict detection
  (a file changed elsewhere is never overwritten), revert/reload, unsaved-changes guards, clickable wikilinks.
  `studyfrog-md::blocks` (preview model, pulldown-cmark), `studyfrog-service::document`, `resolve_title`
  (TASK-0040, TASK-0041).
- Rehearsal screen: keyboard-first (Space, 1-4, S, P, Esc), pause that hides the question, flip and type-answer
  modes, look-ahead content loading, end-of-run summary; every step saved through the service (spaced runs update
  the schedule, practice-only runs do not). Service `start_run`, `load_item_content`, `StepContext`; crashed runs
  are recovered when a vault opens. 15 tests (TASK-0039). EPIC-008 (UI shell) complete.
- Run setup screen: presets (spaced, everything in order, shuffled, hardest first, not seen for longest),
  all orders, filters (kind, stack, tags, state, difficulty), limit, requeue, and an explicit "update my
  schedule" switch; live count of matching items from the index. 6 tests (TASK-0073).
- Stack browser: choose a stack or the inbox, filter its items by title, create stacks and cards from the UI.
  Service use cases `create_stack`/`create_card` (never overwrite, validated, reindexed at once), in-house ULID
  composition with OS randomness, `list_items`; 14 new tests incl. a real-vault build-a-stack flow
  (TASK-0038).
- First desktop app (EPIC-008 so far): eframe/egui window with a vault chooser (remembers the last vault),
  library overview (stacks, counts, inbox, problems), navigation and settings view; pure
  `update(State, Msg) -> (State, Vec<Effect>)` with a worker-thread executor and 1 s change polling;
  21 tests incl. headless egui view tests and an accessibility audit; `--smoke-frames N` for CI
  (TASK-0036, TASK-0037).
- `studyfrog-service::session` (open a vault, index it, poll it), `studyfrog-store::overview` (stacks, counts,
  problems), per-user `app.toml` (`studyfrog-config`, `studyfrog-store::app_config`), `poll::prime`.
- `studyfrog-config` (ADR 0007, Proposed): pure parsing and rendering of `studyfrog.toml` (vault defaults:
  scheduler, retention, interval cap, FSRS weights) and `_stack.toml` (scheduler, retention, daily limits);
  unknown keys are errors with line numbers (TASK-0075).
- Settings pipeline: `_stack.toml` is indexed (`stack_identity`, `stack_config`, view `stack_setting`), loaded into
  the library, and included in polling; schema v3 (TASK-0075).
- SM-2 in the service: each item is scheduled by its stack's scheduler (default from `studyfrog.toml`),
  stack retention overrides apply to FSRS, switching a stack keeps items scheduled (TASK-0076).
- Per-stack daily new/review limits for spaced runs, counted from the review history with an injected day window;
  practice-only runs do not count (TASK-0077).
- Run filters are persisted with the run and round-trip exactly (TASK-0074). EPIC-018 complete.
- `studyfrog-service` (application layer, ADR 0006 Proposed): `apply_rating` (pure), `record_step`,
  `begin_run`, `recover_runs`, `new_run`/`candidates`; end-to-end tests over a real vault and database
  (spaced run updates the schedule; practice-only run leaves it untouched; due items return most-overdue
  first; crash recovery) (TASK-0033).
- `studyfrog-store`: schema v2 (run plan columns, in-place upgrade tested), run and event persistence
  with crash recovery, review and FSRS state storage, item positions (TASK-0072, TASK-0033).
- `studyfrog-core`: event name/payload encode and decode. Phase M3 complete.
- Queue builders in `studyfrog-rehearsal`: filters (kind, stack, tags case-insensitive incl. non-ASCII,
  state, difficulty), orders (spaced with daily budgets, sequential, seeded shuffle, hardest/easiest,
  stalest, lowest predicted recall), length cap; in-house SplitMix64 shuffle; 13 tests incl. an
  independent filter oracle; benchmark: engine step 23 ns, 10k-candidate queue 82 us to 3.9 ms
  (TASK-0034, 0069, 0070, 0035).
- Run plans in `studyfrog-core`: orders (due-first, sequential, shuffled, hardest/easiest, stalest,
  lowest predicted recall), filters, schedule effect (update vs practice-only), requeue-on-Again, limit
  (TASK-0068). Rehearsal is not only spaced repetition.
- `studyfrog-rehearsal`: pure run state machine (`step`), injected clock, latency excluding paused
  time, requeue with cap; 9 tests incl. an invariant property (TASK-0030, 0031, 0032, 0071).
- `studyfrog-store`: vault walker, in-house content hash, stack ownership by folder, non-fatal scanner
  with duplicate-id detection, `Library` builder (inbox stack for orphans), incremental reindex
  (stat-only for unchanged files, rename-safe, duplicate-healing), polling change detection with
  settle window; 17 tests and a 10k-note benchmark (cold index 0.57 s). EPIC-006 and phase M2
  complete (TASK-0027..0029).
- `studyfrog-store`: SQLite schema v1 (derived index + authoritative history), transactional
  migration runner, WAL/foreign-key pragmas, 6 tests (TASK-0025, TASK-0026).
- `studyfrog-md`: in-house ULID and RFC 3339 codecs, front-matter fence/key-block splitter, typed
  parser and renderer for stack/note/card/question with verbatim unknown keys, `???` splitter,
  wikilink extraction, duplicate-id check; 25 tests incl. 1000-case round-trip property
  (TASK-0020..0024). EPIC-005 complete.
- `studyfrog-srs`: SM-2 (`sm2_next`, ease/interval rules from the original), `Sm2State` in core, and a
  criterion benchmark (100k cards: FSRS 2.9 ms, SM-2 0.72 ms); FSRS budget tightened to 10 ms
  (TASK-0018, TASK-0019). EPIC-004 and phase M1 complete.
- `studyfrog-srs`: FSRS-6 (`initial_state`, `next_state`, `retrievability`, intervals, Anki-style
  deterministic fuzz); `MemState` in core; golden test reproducing the py-fsrs reference intervals and
  7 property tests (TASK-0016, TASK-0017).
- `studyfrog-core`: `Run`, `RunMode`, `RunState`, `RunEvent`, `EventKind`, `Latency` and persisted-name
  functions, 2 tests (TASK-0015). EPIC-003 (core domain types) complete.
- `studyfrog-core`: `Rating` (persisted values 1-4), `Review`, `rating_value`/`rating_from_value`,
  3 tests (TASK-0014).
- `studyfrog-core`: `Meta`, `Stack`, `Note`, `Card`, `Question`, `UnixMs` and `Library` (one arena per
  kind, no `Item` enum), plus a test (TASK-0013).
- `BOARD.md`: Kanban view generated from issue front-matter by `scripts/board.sh`; WIP limit 3;
  `scripts/check.sh` fails when it is stale.
- `CHANGELOG.md` and `CLAUDE.md` (working rules: keep board and changelog current).
- `studyfrog-core`: `Uid`, typed arena index `Idx<T>`, append-only `Arena<T>` with free
  functions, 7 tests (TASK-0012).
- M1 issues: EPIC-003/004, STORY-0005..0008, TASK-0012..0019.
- `tools/repo-lint` (standalone, `syn` + `serde-saphyr`): inherent-`impl` scan including macro
  bodies, and issue front-matter validation (TASK-0006).
- `scripts/check-issues.sh`, `scripts/check-deps.sh`, `scripts/check-impls.sh`,
  `scripts/linecount.sh`, `scripts/check.sh` (TASK-0003..0005).
- Documentation library: vision, architecture, design principles, glossary, ADRs 0001-0005,
  roadmap M0-M8 with phase files, specs, research, contributing guides (TASK-0007..0011).
- M0 issues: EPIC-001/002, STORY-0001..0004, TASK-0001..0011.
- Cargo workspace with 12 documented, empty crates and shared lints (TASK-0001, TASK-0002).
- `rustfmt.toml`, `clippy.toml`, `rust-toolchain.toml` (1.99.1), `.gitignore`.

### Changed
- Queues are built from the index (`studyfrog_store::overview::load_indexed_items`) instead of a parsed
  `Library`; `new_run` no longer takes a library.
- Stack settings (`daily_*`, `srs`) are no longer `_stack.md` front-matter; they moved to `_stack.toml`
  (ADR 0007). `StackDoc` now carries only identity, description and body; core `Stack` has `settings`.
- ADR 0005 amendment accepted: no `gray_matter`; in-house fence splitter plus `serde-saphyr`
  (TASK-0021 retitled). New design principle P10: build instead of buy when we can do better.
- ADRs 0002-0005 accepted (GUI: egui/eframe; persistence: rusqlite + Markdown files;
  SLM: candle, gated on M6 spike; Markdown schema).
- ADR 0005: YAML crate decided (`serde-saphyr`); amendment to drop `gray_matter` is Proposed.
- `docs/specs/srs-algorithm.md` pins FSRS-6 with default weights.
- Research citations verified where possible and marked `[v]`/`[u]`; benchmark budgets
  documented as provisional with rationale.
- Default branch is `main`.

### Removed
- Placeholder hello-world package and the unverified `repository` manifest field.

### Fixed
- Graph layout never came to rest because the repulsion jumped to zero at its cutoff (nodes flickered in and out of
  range); a smooth window fixes it. Found by measuring energy, not by a failing test.
- Board/issue bookkeeping: TASK-0016, STORY-0007 and EPIC-004 were still `backlog` after M1. `repo-lint`
  now rejects a parent whose children are all done but is not, a done parent with open children, and a
  done item with unchecked acceptance criteria.
- `scripts/linecount.sh` no longer miscounts the `wc` total line on non-English locales.
