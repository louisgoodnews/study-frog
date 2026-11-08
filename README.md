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
- **Released: 0.5.0** (2026-11-08), part of the planned phase M12: an application home with rotating logs
  (`~/.local/share/studyfrog`, `studyfrog --print-paths`, `studyfrog logs`; `STORY-0122`); a Model Context Protocol
  server for AI tools (`studyfrog --mcp`, read-only; `STORY-0117`, `STORY-0118`, `STORY-0120`); Google Gemini and
  DeepSeek as assistant providers (`STORY-0083`, `STORY-0084`); four more prompts, categories, options and a Prompts
  screen (`STORY-0088` to `STORY-0092`, `TASK-0233`); 32 achievement tiers with a per-stack view (`STORY-0086`,
  `STORY-0087`); the reports as CSV (`STORY-0098`); and a fix for blurred PDF pages (`BUG-0010`, `STORY-0085`). Notes:
  [docs/RELEASE_NOTES_0.5.0.md](docs/RELEASE_NOTES_0.5.0.md). Earlier releases:
  [0.4.3](docs/RELEASE_NOTES_0.4.3.md), [0.4.2](docs/RELEASE_NOTES_0.4.2.md), [0.4.1](docs/RELEASE_NOTES_0.4.1.md),
  [0.4.0](docs/RELEASE_NOTES_0.4.0.md), [0.3.1](docs/RELEASE_NOTES_0.3.1.md), [0.3.0](docs/RELEASE_NOTES_0.3.0.md),
  [0.2.0](docs/RELEASE_NOTES_0.2.0.md), and [0.1.0](docs/RELEASE_NOTES_0.1.0.md).
- **Planned next (rest of phase M12)** — see **[PLANNED_FEATURES.md](PLANNED_FEATURES.md)**:
  - **Exam run mode (`EPIC-033`)**: strict one-way progression, mandatory written answers, semantic comparison, and
    percentage grading (`60%` pass threshold `D`/`4.0`, `5%` bands up to `95%–100%` `A+`/`1+`).
  - **Item keys & colorization (`EPIC-032`)**: sequential keys (`STACK_1`, `FLASHCARD_500`) and an item color picker
    with viewer background tinting (the CSV export of the reports is in 0.5.0).
  - **Extensibility, Python API, Gemini/DeepSeek & PDF optimization (`EPIC-028`)**: out-of-process JSON-RPC Python
    scripting, and the rest of the PDF viewer performance work (a real-window 60 fps check, eviction by use).
  - **Behavioral learning analytics, FSRS optimizer & workload forecasting (`EPIC-034`)**: chronobiological hour-of-day
    & weekday telemetry, within-session cognitive fatigue detection, multi-signal leech & interference pair diagnostics,
    FSRS state-simulation forecasting in study minutes with an Exam Deadline Pacer, Workload-vs-Retention (CMRR)
    simulation, and stability-tier empirical forgetting curves with True Retention.
  - **Rich active-recall formats, inline note extraction & focus workflows (`EPIC-035`)**: inline Cloze deletions
    (`{{c1::answer::hint}}`), Image Occlusion cards for diagrams/PDF figures, one-click card extraction from Markdown
    notes (`Concept :: Definition` / `Q: / A:`), contrastive side-by-side drills for confused items, and a top-bar
    Pomodoro focus timer integrated with the Learning Diary.
  - **Evidence-based UI aesthetics, perceptual color science & UX refinement (`EPIC-036`)**: OKLCH iso-luminant palette
    calibration, WCAG 3 APCA perceptual contrast verification, 60–75 cpl typographic reading measure, low-entropy
    surface elevation over nested borders, and a distraction-free Zen focus mode.
  - **State-of-the-art Model Context Protocol (MCP) server, client host & registry (`EPIC-037`)**: pure JSON-RPC 2.0 MCP
    engine with a plain-data function-pointer registry (`McpRegistry`), `studyfrog-app --mcp` (`stdio` & localhost
    Streamable HTTP) exposing `studyfrog://` resources, annotated search/analytics/staged-authoring tools, and native
    MCP prompts, plus an MCP Client Host in the built-in Assistant (0.5.0 has the engine, read-only resources and search, and the stdio server).
  - **CLI control protocol for AI agents & `~/.local/` application files (`EPIC-038`)**: interactive CLI control
    protocol (`studyfrog-app ctl` / `--control-stdio`) giving AI agents full operational access to all screens,
    rehearsal/exam runs, state snapshots, and real-time console logs (the logs and paths are in 0.5.0).
- **Verification:** all 10 automated repository gates in `scripts/check.sh` (formatting, `repo-lint`, `clippy`, unit/integration/UI tests, doc-tests, 250-line file limit, crate layering, zero inherent `impl` blocks, issue lint, and `BOARD.md` sync) pass. OCR for scanned PDFs, cloud sync, and non-Linux platforms are not planned for now.

Live work: [BOARD.md](BOARD.md) and [priority-list.md](priority-list.md). Feature reference:
[FEATURES.md](FEATURES.md) and [PLANNED_FEATURES.md](PLANNED_FEATURES.md). Where the project stands: [STATUS.md](STATUS.md). History:
[CHANGELOG.md](CHANGELOG.md). Docs: [docs/README.md](docs/README.md).

## Install
Linux on x86_64 with OpenGL and X11 or Wayland. Download from the
[releases](https://github.com/louisgoodnews/study-frog/releases):

```sh
# AppImage: one file
sha256sum -c StudyFrog-0.5.0-x86_64.AppImage.sha256
chmod +x StudyFrog-0.5.0-x86_64.AppImage && ./StudyFrog-0.5.0-x86_64.AppImage

# or the tarball
sha256sum -c StudyFrog-0.5.0-x86_64.tar.gz.sha256
tar -xzf StudyFrog-0.5.0-x86_64.tar.gz && ./StudyFrog-0.5.0/bin/studyfrog
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
