# StudyFrog 0.7.0

Timetable schedule planner for lessons and lectures, calendar daily and weekly schedule views, expanded markdown macro library with escaping syntax, enhanced markdown list syntax, stylized Breviceps macrops brand mascot icon, UX research and prototype of floating burger navigation, scriptrunner development environment with internal automation API, profile and vault switcher overhaul, file navigator and dialogue overhaul, and expandable comments drawer during rehearsal and exam modes. The full list of changes is in `CHANGELOG.md` under `[0.7.0]`.

## What is new

- **Timetable Schedule Planner (`STORY-0141`, `EPIC-044`).** Students and learners can organize their recurring academic schedule directly in StudyFrog. Timetable entries record subject/course, weekday, start and end times, room/location, instructor, and linked flashcard stack. Timetable entries automatically project into recurring sessions on the calendar and notification system, allowing direct rehearsal triggering from class cards.
- **Calendar Daily & Weekly Schedule Views (`STORY-0142`, `EPIC-044`).** The calendar now provides an interactive segmented view mode switcher (`Month | Week | Day | Timetable`):
  - **Week View:** 7-day hourly grid from 06:00 to 22:00 displaying lectures, appointments, tasks, and study sessions with hover popovers and day selection.
  - **Day View:** Single-day hourly agenda with detailed session cards, study workload summaries, and one-click session creation.
- **Comprehensive Markdown Macro Library & Escaping Syntax (`STORY-0143`, `STORY-0144`, `EPIC-045`).** Extends markdown document authoring with built-in macros (`{rule}`, `{header:level}`, `{arrow_right}`, `{arrow_left}`, `{graph:x,y,z}`, `{hide}`, `{panel}`, `{link}`, `{bold:"..."}`, `{italic:"..."}`). Added escaping syntax `{#macro}` to suppress expansion and display macro tokens literally in code examples and prompts.
- **Markdown Viewer List Syntax Enhancements (`STORY-0145`, `EPIC-046`).** The markdown parser and document viewer now render unordered lists using `-` and `*`, ordered decimal lists (`1.`, `2.`), and alphabetical ordered lists (`a)`, `b.`, `c.`) with consistent indentation and typography.
- **Stylized *Breviceps macrops* Mascot Icon (`STORY-0146`, `EPIC-047`).** Replaced generic geometric markers with a custom vector-drawn *Breviceps macrops* (desert rain frog) mascot across the top bar, featuring its signature globular silhouette, soulful forward-facing eyes, and studious expression rendered via crisp scalable vector primitives matching light and dark OKLCH palettes.
- **Floating Burger Navigation Research & Working Prototype (`STORY-0147`, `EPIC-047`).** Authored comprehensive UX evaluation (`docs/research/floating-burger-navigation.md`) analyzing spatial ergonomics, Fitts's law, reading measure protection, and Zen mode synergy. Implemented a collapsible floating action burger trigger in `crates/studyfrog-ui` providing quick 1-click access to all application screens.
- **ScriptRunner Development Environment & Internal Python API (`STORY-0148`, `EPIC-048`).** Expanded the Console into an in-app ScriptRunner environment with multi-line code editor, script template recipes (vault stats, bulk tagger, deck creator), stdout/stderr output stream, and authored the internal `studyfrog` Python API architecture specification (`docs/research/scriptrunner-and-internal-api.md`).
- **Profile & Vault Switcher Overhaul (`STORY-0138`, `EPIC-043`).** Overhauled profile and vault management dialogs with elevated card layouts, active status badges, unambiguous paths, profile avatars, and safe creation/deletion workflows.
- **File Navigator & In-App File Dialogue Overhaul (`STORY-0139`, `EPIC-043`).** Modernized file browser dialog with tree indent guides, file-type icons, inline real-time filtering, path breadcrumbs, and shortcut bookmarks.
- **In-Rehearsal & Exam Expandable Comments Drawer (`STORY-0140`, `EPIC-043`).** Added an expandable side drawer during active rehearsal and exam runs allowing learners to inspect, search, and add item annotations without interrupting study flow.
- **Markdown & Note Refinements:**
  - High-contrast bold text in markdown viewer (`BUG-0017`).
  - Strict preservation of code block indentations and tab stops (`BUG-0018`).
  - Standalone notes: notes being inside stacks is now optional (`BUG-0019`).

## Verification
- **Automated quality gates:** All 10 repository checks in `scripts/check.sh` pass cleanly on release commit:
  - Formatting (`cargo fmt --all --check`)
  - Tool linters (`repo-lint`)
  - Strict compiler lints (`cargo clippy --workspace --all-targets -- -D warnings`)
  - Full test suite (`cargo test --workspace`) across all 21 crates
  - Line count guardrail ($\le 250$ lines per `.rs` file)
  - Layering and dependency direction
  - Architecture guardrail (zero inherent `impl` blocks)
  - Issue metadata validation
  - Board consistency (`BOARD.md` 0 backlog, 267 done)

## Install
StudyFrog is for Linux on x86_64. It requires OpenGL, and X11 or Wayland, from your system.

### AppImage
```sh
sha256sum -c StudyFrog-0.7.0-x86_64.AppImage.sha256
chmod +x StudyFrog-0.7.0-x86_64.AppImage
./StudyFrog-0.7.0-x86_64.AppImage
```

### Tarball
```sh
sha256sum -c StudyFrog-0.7.0-x86_64.tar.gz.sha256
tar -xzf StudyFrog-0.7.0-x86_64.tar.gz
cd StudyFrog-0.7.0
./bin/studyfrog
```

### Upgrading from 0.6.1
SQLite store migration V28 (`timetable_entry`) executes automatically upon opening your vault. All existing notes, cards, and review histories remain fully compatible.
