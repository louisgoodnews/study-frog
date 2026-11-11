# StudyFrog 0.7.1

Conversational chat interface in Assistant, universal importer integration in items view, file launching in folders view, dedicated item deletion modal popup, difficulty-weighted metacognitive calibration, window icon integration, ScriptStudio & FrogScript rebranding and language specification, and critical fixes for calendar rehearsal setup and due item calculation. The full list of changes is in `CHANGELOG.md` under `[0.7.1]`.

## What is new

- **Assistant Conversational Chat Interface.** Added a dedicated "Chat" tab to the Assistant screen with conversational message flow, auto-scrolling message bubbles, real-time message sending on Enter or Send button click, clear conversation action, and full integration with configured local and remote LLM providers.
- **Universal Importer in Items and Stack Views.** The "Import…" button in Library items and stack views now routes directly to the universal importer, enabling import of notes, stacks, questions, flashcards, CSV/TSV spreadsheets, Obsidian vaults, and Anki packages.
- **Folders Viewer Document Launching.** Double-clicking or right-clicking "Open" on PDF documents in the Folders tree navigator directly opens them in the built-in PDF viewer.
- **Dedicated Item Deletion Modal Dialog.** Replaced the inline deletion confirmation bar with a centered modal popup with backdrop dimming and explicit Delete Item and Cancel buttons.
- **Item Difficulty-Weighted Metacognitive Calibration.** Self-assessment calibration scoring now incorporates item difficulty to prevent false overconfidence penalties on difficult materials.
- **Window Icon Integration.** Embedded stylized *Breviceps macrops* vector logo icon into application window and desktop packaging.
- **ScriptStudio & FrogScript Rebranding and Specification (`STORY-0149`, `STORY-0150`, `EPIC-049`).** Rebranded ScriptRunner console to ScriptStudio to respect external trademarks, and authored the FrogScript DSL language specification (v1.0).
- **Calendar & Dashboard Due Item Alignment:**
  - Aligned day-end timestamp cutoffs so items scheduled for today are counted consistently across both Dashboard and Calendar.
  - Fixed review queue admission when initiating rehearsal from calendar and timetable cards, resolving 0-item selection and adding recursive substack inclusion.

## Verification

- **Automated quality gates:** All repository checks in `scripts/check.sh` pass cleanly on release commit:
  - Formatting (`cargo fmt --all --check`)
  - Tool linters (`repo-lint`)
  - Strict compiler lints (`cargo clippy --workspace --all-targets -- -D warnings`)
  - Full test suite across all 21 crates
  - Line count guardrail ($\le 250$ lines per `.rs` file)
  - Layering and dependency direction
  - Architecture guardrail (zero inherent `impl` blocks)
  - Issue metadata validation
  - Board consistency

## Install

StudyFrog is for Linux on x86_64. It requires OpenGL, and X11 or Wayland, from your system.

### AppImage

```sh
sha256sum -c StudyFrog-0.7.1-x86_64.AppImage.sha256
chmod +x StudyFrog-0.7.1-x86_64.AppImage
./StudyFrog-0.7.1-x86_64.AppImage
```

### Tarball

```sh
sha256sum -c StudyFrog-0.7.1-x86_64.tar.gz.sha256
tar -xzf StudyFrog-0.7.1-x86_64.tar.gz
cd StudyFrog-0.7.1
./bin/studyfrog
```

### Upgrading from 0.7.0

All existing vaults, notes, cards, and review histories remain fully compatible.
