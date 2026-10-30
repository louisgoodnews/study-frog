# Changelog

All notable changes to StudyFrog are recorded here, newest first.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). The project is pre-release, so
everything is under `Unreleased` until the first tagged version. Each entry names the work item
(`TASK-xxxx`) it belongs to where one exists.

## [Unreleased]

### Added
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
- `scripts/linecount.sh` no longer miscounts the `wc` total line on non-English locales.
