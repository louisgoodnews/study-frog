# StudyFrog 0.6.1

In-rehearsal item editing, intermittent typed active recall sampling, item comments notebook tab, system-driven background file surveys, interactive AI content generation prompts, end-of-run placement for cards rated Again, and realistic metacognitive calibration accuracy baselines. The full list is in `CHANGELOG.md` under `[0.6.1]`.

## What is new
- **In-Rehearsal Card & Item Quick Editing.** Learners can now edit cards, questions, and notes directly during an active rehearsal run without abandoning or restarting their session. Triggered via keyboard shortcut (`Ctrl+E` / `Cmd+E` / `F2`) or on-screen `✏ Edit` buttons on the card and top strip, the editor pauses the session and elapsed timer. Saving persists markdown changes to disk, re-indexes the document, and refreshes the current card's front/back blocks live in place.
- **Intermittent Typed Active Recall Sampling (`STORY-0136`, `EPIC-040`).** Rehearsal setup introduces a `Typed answer sampling (0-100%)` slider. During rehearsal runs, the engine randomly alternates between typed answer prompts and traditional flip cards according to the selected probability, keeping study sessions engaging and unpredictable. Notes are automatically exempted from typed prompts, and a mode indicator on the card header clearly identifies whether an answer must be typed or recalled mentally.
- **Item Comments Notebook Tab (`STORY-0135`, `EPIC-039`).** The Item Details notebook now includes a dedicated `Comments` tab. Learners and AI assistants can attach timestamped annotations, hints, or feedback to any stack, card, question, or note without cluttering the primary prompt or body text, complete with author badges and deletion controls.
- **System-Driven Modularity & Immediate File Surveys (`EPIC-041`).** File surveys, folder tree scanning, and library index updates now execute automatically upon vault open, vault switch, and dashboard navigation. Existing files (such as newly added PDFs or notes) appear immediately on the dashboard and in search results without requiring the user to open the Folders/Files screen first.
- **Interactive Parameter-Guided Content Generation Prompts (`STORY-0137`, `EPIC-042`).** Added the `interactive_extract` prompt to the Prompt Library under `Create content`. Instead of blind one-shot generation, the prompt instructs AI models to conduct an intake interview (asking for preferred topic scope, language, difficulty tier, and card/note format) before generating structured StudyFrog CSV import blocks.
- **Rehearsal Lapse Queue Reordering.** When rating a card `Again` during rehearsal, the card is now automatically moved to the last position of the active run queue instead of reappearing prematurely. This ensures learners complete the full rehearsal cycle before re-testing forgotten material.
- **Realistic Metacognitive Calibration Accuracy Baseline (`BUG-0016`).** Replaced the naive binary recall count (`recalled / total`) in the self-assessment calibration report with a scientifically grounded baseline accuracy calculation. The model weights item novelty and familiarity ($0.35 + 0.65 \times \text{familiarity}$), rating grades (`Easy` 1.0, `Good` 0.85, `Hard` 0.50, `Again` 0.0), and reading latency relative to content length and expected reading speed. This eliminates distorted 100% measured accuracy figures on novel cards and restores meaningful calibration gap insights.

## Verification
- **Automated quality gates:** All repository checks in `scripts/check.sh` (`cargo fmt --check`, `repo-lint`, `cargo clippy -D warnings`, workspace unit/integration/UI tests, linecount $\le 250$, dependency direction, zero inherent `impl` blocks, issue front-matter lint, and `BOARD.md` sync) pass on the release commit.

## Install
StudyFrog is for Linux on x86_64. It needs OpenGL, and X11 or Wayland, from your system; nothing else. Download the
files of your choice from the assets of this release.

### AppImage (one file, nothing to unpack)
```sh
sha256sum -c StudyFrog-0.6.1-x86_64.AppImage.sha256   # prints "OK" when the download is intact
chmod +x StudyFrog-0.6.1-x86_64.AppImage
./StudyFrog-0.6.1-x86_64.AppImage
```

### Tarball (`.tar.gz`)
```sh
sha256sum -c StudyFrog-0.6.1-x86_64.tar.gz.sha256     # prints "OK" when the download is intact
tar -xzf StudyFrog-0.6.1-x86_64.tar.gz
cd StudyFrog-0.6.1
./bin/studyfrog
```

### Upgrading from 0.6.0
Nothing to migrate: open your vault as before.
