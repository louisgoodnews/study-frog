# Status

**Phase:** M1 Core domain (in progress; M0 exit criteria met). **Date:** 2026-10-30. **Branch:** `main`.

## What exists
- Cargo workspace with 12 empty, documented crates (`crates/studyfrog-*`); `cargo check` passes.
- `docs/`: vision, architecture, principles, glossary, ADRs 0001-0005 (all Accepted),
  roadmap M0-M8 (17 epics, 32 stories, 67 tasks planned), specs, research, contributing guides.
- `docs/issues/`: M0 seeded (EPIC-001/002, STORY-0001..0004, TASK-0001..0011).
- Guardrails in `scripts/`: `check.sh` (fmt, clippy, test, line count, layering, no-OOP),
  `linecount.sh`, `check-deps.sh`, `check-impls.sh`, `check-issues.sh`. Each check was shown to
  fail on a violation. `tools/repo-lint` (standalone, syn + serde-saphyr) backs the last two.
- Config: `rustfmt.toml`, `clippy.toml`, `rust-toolchain.toml` (1.99.1), `.gitignore`.
- Domain code so far: only `studyfrog-core` ids and arena (data + free functions; the sole `impl`
  blocks are the `Idx<T>` trait impls, each delegating to the raw index).

## Flags and known gaps
- **Needs your decision:** ADR 0005 amendment (Proposed) to drop `gray_matter` in favour of an
  own fence splitter plus `serde-saphyr` (YAML crate itself is decided and validated).
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
**TASK-0014** (Review and Rating types) then **TASK-0015** (Run and event types), both `ready`.
Done in M1 so far: TASK-0012 (`Uid`, `Idx<T>`, `Arena<T>`), TASK-0013 (`Meta`, `Stack`, `Note`,
`Card`, `Question`, `Library` with one arena per kind, `UnixMs`). STORY-0005 is done.
Live work state is always in [BOARD.md](BOARD.md).
