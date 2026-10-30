# Changelog

All notable changes to StudyFrog are recorded here, newest first.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). The project is pre-release, so
everything is under `Unreleased` until the first tagged version. Each entry names the work item
(`TASK-xxxx`) it belongs to where one exists.

## [Unreleased]

### Added
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
