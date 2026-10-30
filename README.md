# StudyFrog

A Linux-native desktop learning app: Markdown note graph, SRS flashcards, timed rehearsal
runs, planning calendar, kanban board, achievements, and local-SLM answer comparison.

Written in Rust as a functional, data-oriented Cargo workspace (no OOP idioms).

**Status:** M0 bootstrap, scaffolding and docs only. See [STATUS.md](STATUS.md) and
[docs/README.md](docs/README.md).

## Layout

- `crates/` workspace members (`studyfrog-*`)
- `docs/` vision, architecture, ADRs, roadmap, specs, issues
- `scripts/` guardrail scripts (`check.sh`, `linecount.sh`)

## Dependency direction

`studyfrog-app → studyfrog-ui → domain crates → studyfrog-core`. Domain crates never depend
on the UI.

## Run
```
cargo run -p studyfrog-app                 # opens the window
cargo run -p studyfrog-app -- --smoke-frames 30   # draws 30 frames and exits (CI, under xvfb-run)
scripts/check.sh                           # the full gate
```

