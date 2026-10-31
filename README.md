# StudyFrog

A Linux-native desktop learning app: a Markdown note vault with links, SRS flashcards, timed rehearsal runs, a
planning calendar, a kanban board, achievements, reports, offline answer comparison and an optional AI assistant.

Written in Rust as a functional, data-oriented Cargo workspace (no OOP idioms).

**Status:** version 0.1.0. See [docs/RELEASE_NOTES_0.1.0.md](docs/RELEASE_NOTES_0.1.0.md) for what it does and what
is not verified, [STATUS.md](STATUS.md) for where the project stands and [docs/README.md](docs/README.md) for the docs.

## Install
Download `StudyFrog-0.1.0-x86_64.AppImage` from the release, `chmod +x` it and run it. Or build:
```
cargo run -p studyfrog-app --release          # opens the window
```
Needs Linux, OpenGL and (for file dialogs) an XDG desktop portal.

## Layout
- `crates/` workspace members (`studyfrog-*`)
- `docs/` vision, architecture, ADRs, roadmap, specs, issues
- `scripts/` guardrail scripts (`check.sh`, `board.sh`, `package-appimage.sh`)

## Dependency direction
`studyfrog-app → studyfrog-ui → studyfrog-service → domain crates → studyfrog-core`. Domain crates never depend on
the UI; only `studyfrog-net` links an HTTP library and only the app links `studyfrog-net` (`scripts/check-deps.sh`).

## Develop
```
scripts/check.sh                                  # the full gate: fmt, clippy, tests, size, layering, issues
cargo run -p studyfrog-app -- --smoke-frames 30   # draws 30 frames and exits (CI, under xvfb-run)
```
Read [CLAUDE.md](CLAUDE.md) and [docs/DESIGN_PRINCIPLES.md](docs/DESIGN_PRINCIPLES.md) first. Releasing:
[docs/contributing/RELEASING.md](docs/contributing/RELEASING.md).

## License
Dual-licensed under MIT and Apache-2.0 ([LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE)).
