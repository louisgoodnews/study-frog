# StudyFrog

A Linux-native desktop learning app. Your notes, flashcards, questions, and learning diary entries are plain Markdown
files in a folder (the *vault*); StudyFrog adds the learning system on top: spaced-repetition scheduling, timed
rehearsal, a planning calendar, Kanban boards, a learning diary, PDF reading and export, reports, and an optional AI
assistant with a parameterized prompt library. Everything works offline and your files stay readable without it.

Written in Rust as a functional, data-oriented Cargo workspace (no OOP idioms).

For a complete guide to everything StudyFrog offers today, see **[FEATURES.md](FEATURES.md)**. For upcoming roadmap epics and stories, see **[PLANNED_FEATURES.md](PLANNED_FEATURES.md)**.

## What it does
- **Vault of Markdown** with links between notes (`[[wikilinks]]`), a link graph, cards, questions, notes, diary
  entries, standalone and nested stacks (`Move stack…` between parent and top-level), cross-stack references
  (`referenced_by`), tags, priorities, difficulties, statuses, and a compact Jira-inspired Vaults switcher.
- **Folders & Items views**: an Obsidian-style folder tree (create, inline `F2` rename with link rewriting, drag &
  drop, convert folder to/from stack) alongside a sortable library table with standard search, a SQL-like query
  language, saved filters, and bulk edits with undo. Each stack opens a nine-tab workspace: Summary, List, Backlog,
  Roadmap, Board, Calendar, Reports, Sub-stacks, and Goals.
- **Rich Editor & Details Notebook**: distraction-free Write, Preview, and Split Markdown editing (tables, syntax-highlighted
  code, underline, palette colors, `[TOC]`, smart lists, text macros, conflict-safe auto-save, and item deletion with
  undo) paired with a four-tab Details notebook (`Properties`, `Schedule`, `Relations`, `Learning`).
- **Rehearsal** in any order, with a free run, a countdown or a count-up; independent FSRS-6 or SM-2 scheduling per
  stack; typed answers compared offline (a small language model if installed, else word matching); a scrollable
  post-run review with suggested ratings and an inline diary prompt.
- **Familiarity** (`0.0` to `1.0` per item and stack) that shows what needs attention, steers the queue, and tempers
  the schedule.
- **Learning Diary & Calibration** with 13 standard 1–5 self-assessment scales, custom scales, list and calendar views,
  and calibration reports comparing subjective ratings against measured retention.
- **Calendar**, **Board**, **Achievements**, and **Reports** on retention, answer times, forecast, familiarity,
  self-assessment calibration, and runs.
- **PDFs** you can read, search, highlight, rearrange, and turn into notes and cards—plus native **Markdown-to-PDF
  export** for single items, selections, and full stacks with live preview.
- **Import and export** as Markdown, CSV, TSV, JSON, TOML, or RON (with smart recovery for unquoted commas, multiline
  fields, and escaped newlines in AI-generated CSVs); import from Anki packages (`.apkg`) and Obsidian vaults.
- **Prompt Library & Assistant**: ready-made AI prompts (Flashcards, Questions, Whole Stack, Notes, Summary, Critique)
  with one-click copy, Markdown export, or optional assistant execution where all proposed changes require explicit
  approval.
- **Appearance & Help**: dark, light or system theme, UI/text scaling, density, accent colours, high contrast, reduced
  motion, and built-in Help (`F1`) with how-to guides, FAQ, and an interactive tutorial.

## Status
- **Released: 0.6.0** (2026-11-09): Exam mode with grading, item keys and colours, learning analytics (habits, leeches,
  workload forecast, true retention), cloze deletions, image occlusion, cards from notes, a contrastive drill and a focus
  timer, a control protocol with a Python client, scripts and a Console, the rest of the Model Context Protocol work (reports,
  authoring with staging, an HTTP transport and a client for external tool servers), OKLCH/APCA palettes, a type scale and
  Zen mode, and a sharper PDF viewer. Notes: [docs/RELEASE_NOTES_0.6.0.md](docs/RELEASE_NOTES_0.6.0.md). Earlier releases:
  [0.5.0](docs/RELEASE_NOTES_0.5.0.md), [0.4.3](docs/RELEASE_NOTES_0.4.3.md), [0.4.2](docs/RELEASE_NOTES_0.4.2.md),
  [0.4.1](docs/RELEASE_NOTES_0.4.1.md), [0.4.0](docs/RELEASE_NOTES_0.4.0.md), [0.3.1](docs/RELEASE_NOTES_0.3.1.md),
  [0.3.0](docs/RELEASE_NOTES_0.3.0.md), [0.2.0](docs/RELEASE_NOTES_0.2.0.md), and [0.1.0](docs/RELEASE_NOTES_0.1.0.md).
- **Open points** — see **[PLANNED_FEATURES.md](PLANNED_FEATURES.md)**: STORY-0114 (a reading measure in pixels or in
  characters) and STORY-0124 (control protocol commands for every screen) are in review.
- **Verification:** all 10 automated repository gates in `scripts/check.sh` (formatting, `repo-lint`, `clippy`, unit/integration/UI tests, doc-tests, 250-line file limit, crate layering, zero inherent `impl` blocks, issue lint, and `BOARD.md` sync) pass. OCR for scanned PDFs, cloud sync, and non-Linux platforms are not planned for now.

Live work: [BOARD.md](BOARD.md) and [priority-list.md](priority-list.md). Feature reference:
[FEATURES.md](FEATURES.md) and [PLANNED_FEATURES.md](PLANNED_FEATURES.md). Where the project stands: [STATUS.md](STATUS.md). History:
[CHANGELOG.md](CHANGELOG.md). Docs: [docs/README.md](docs/README.md).

## Install
Linux on x86_64 with OpenGL and X11 or Wayland. Download from the
[releases](https://github.com/louisgoodnews/study-frog/releases):

```sh
# AppImage: one file
sha256sum -c StudyFrog-0.6.0-x86_64.AppImage.sha256
chmod +x StudyFrog-0.6.0-x86_64.AppImage && ./StudyFrog-0.6.0-x86_64.AppImage

# or the tarball
sha256sum -c StudyFrog-0.6.0-x86_64.tar.gz.sha256
tar -xzf StudyFrog-0.6.0-x86_64.tar.gz && ./StudyFrog-0.6.0/bin/studyfrog
```
The release notes have the full steps (menu entry, per-user install, removing it). To build from source:
```sh
cargo run -p studyfrog-app --release          # opens the window
```
File dialogs need an XDG desktop portal.

## Layout
- `crates/` workspace members (`studyfrog-*`), `examples/` an example extension
- `docs/` vision, architecture, ADRs, roadmap, specs, help pages, issues
- `scripts/` guardrail scripts (`check.sh`, `board.sh`, `package-appimage.sh`)

## Dependency direction
`studyfrog-app → studyfrog-ui → studyfrog-service → domain crates → studyfrog-core`. Domain crates never depend on
the UI; only `studyfrog-net` links an HTTP library and only the app links `studyfrog-net` (`scripts/check-deps.sh`).

## Develop
```sh
scripts/check.sh                                  # the full gate: fmt, clippy, tests, size, layering, issues
cargo run -p studyfrog-app -- --smoke-frames 30   # draws 30 frames and exits (CI, under xvfb-run)
cargo run -p studyfrog-app -- --screenshot home.png --screen home   # a PNG of a screen of the real app
```
Read [CLAUDE.md](CLAUDE.md) and [docs/DESIGN_PRINCIPLES.md](docs/DESIGN_PRINCIPLES.md) first. Releasing:
[docs/contributing/RELEASING.md](docs/contributing/RELEASING.md).

## License
Dual-licensed under MIT and Apache-2.0 ([LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE)).
