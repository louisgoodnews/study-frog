# StudyFrog

A Linux-native desktop learning app. Your notes, flashcards and questions are plain Markdown files in a folder (the
*vault*); StudyFrog adds the learning system on top: spaced-repetition scheduling, timed rehearsal, a planning
calendar, reports, and an optional AI assistant. Everything works offline and your files stay readable without it.

Written in Rust as a functional, data-oriented Cargo workspace (no OOP idioms).

## What it does
- **Vault of Markdown** with links between notes (`[[wikilinks]]`), a link graph, cards, questions, stacks and
  sub-stacks, tags, priorities, difficulties and statuses.
- **Library** as a sortable table of every item with a search bar (standard fields or a SQL-like query language), saved
  filters and bulk changes with undo. A stack opens as a page with tabs: Summary, List, Backlog, Roadmap, Board,
  Calendar, Reports, Sub-stacks and Goals.
- **Rehearsal** in any order, with a free run, a countdown or a count-up; FSRS-6 or SM-2 scheduling per stack; typed
  answers compared offline (a small language model if installed, else word matching); a review after each run.
- **Familiarity** (0.0 to 1.0 per item) that shows what needs attention, steers the queue and tempers the schedule.
- **Calendar** with planned sessions, exams and the reviews the schedule plans, which can be dragged to another day.
- **Board**, **achievements**, and **reports** on retention, answer times, forecast, familiarity and runs.
- **PDFs** you can read, search, highlight, rearrange, and turn into notes and cards.
- **Import and export** as JSON, CSV or Markdown; import from Anki packages and Obsidian vaults, with suggestions from
  what it finds on your computer.
- **Appearance settings**: dark, light or follow the system, interface and text size, density, accent colours, high contrast, reduced motion, the start screen and the remembered window, all applied at once.
- **Help inside the app** (`F1`): how-to pages, FAQ, your own pages in the vault, and a first-steps tutorial.
- **Assistant** (optional): proposes notes, cards and edits that change nothing until you apply them; sends text only
  after you press Send.

## Status
- **Released: 0.3.1** (2026-11-03): fixes and a calmer UI after the first use of 0.3.0: StudyFrog's own title bar, a
  roomier layout with reading columns and aligned rows (library, stack page, Home, rehearsal setup, settings, help),
  a progress bar in the rehearsal, a discard question in the Create dialog and import suggestions. Notes:
  [docs/RELEASE_NOTES_0.3.1.md](docs/RELEASE_NOTES_0.3.1.md). Earlier: [0.3.0](docs/RELEASE_NOTES_0.3.0.md)
  (usability and insight), [0.2.0](docs/RELEASE_NOTES_0.2.0.md) (PDF support) and
  [0.1.0](docs/RELEASE_NOTES_0.1.0.md).
- **Planned: 0.4.0**, a diary of self-reported progress next to the measured results, and the calmer look for the
  screens not yet done (Calendar, Board, Reports, Achievements, Assistant, Graph, the Create dialog).
- **Not verified by hand:** much of the UI was tested headless only; real-window behaviour is checked as it comes up.
  OCR for scanned PDFs, cloud sync and other platforms are not planned for now. The release notes list what each
  version was and was not checked with.

Live work: [BOARD.md](BOARD.md) and [priority-list.md](priority-list.md). Where the project stands:
[STATUS.md](STATUS.md). History: [CHANGELOG.md](CHANGELOG.md). Docs: [docs/README.md](docs/README.md).

## Install
Linux on x86_64 with OpenGL and X11 or Wayland. Download from the
[releases](https://github.com/louisgoodnews/study-frog/releases):

```sh
# AppImage: one file
sha256sum -c StudyFrog-0.3.1-x86_64.AppImage.sha256
chmod +x StudyFrog-0.3.1-x86_64.AppImage && ./StudyFrog-0.3.1-x86_64.AppImage

# or the tarball
sha256sum -c StudyFrog-0.3.1-x86_64.tar.gz.sha256
tar -xzf StudyFrog-0.3.1-x86_64.tar.gz && ./StudyFrog-0.3.1/bin/studyfrog
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
