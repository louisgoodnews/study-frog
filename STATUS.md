# Status

**Phase:** M1 complete; M2 (Markdown and storage) seeded and next. **Date:** 2026-10-30. **Branch:** `main`.

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

## Next task
**M2:** start with TASK-0020 (front-matter schema structs and validation) and TASK-0021 (in-house
fence splitter + serde-saphyr; spike whether it preserves key order). Live state: [BOARD.md](BOARD.md).

## M1 summary
- `studyfrog-core`: `Uid`, `Idx<T>`, `Arena<T>`, `Meta`, `Stack`, `Note`, `Card`, `Question`,
  `Library` (one arena per kind), `Rating`, `Review`, `Run`, `RunEvent`, `Latency`, `MemState`,
  `Sm2State`, `UnixMs`.
- `studyfrog-srs`: FSRS-6 (golden test reproduces py-fsrs intervals; 7 properties), SM-2 (hand-computed
  vectors + properties), deterministic fuzz. Benchmarks: FSRS 2.9 ms / 100k cards, SM-2 0.72 ms.
