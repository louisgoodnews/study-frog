# StudyFrog 1.0.0

The first complete academic workstation release: a planner for homework and deadlines, a single Universal Import entry point with tooltips on every control, an in-app FrogScript wiki and cookbook, Memory Mode with a time attack, a cognitive profile built from timed answers, scenario-based case studies, psychometric math drills, and a mock exam (Altklausur) generator with the German grade scale. Everything runs locally; no telemetry leaves the machine. The full list of changes is in `CHANGELOG.md` under `[1.0.0]`.

Read the [Known limits](#known-limits) before relying on a feature. Several parts of this release are built and tested but still wait for a check on a running window, on the owner's machine, or with a real language model.

## What is new

### Academic planning (EPIC-056, `crates/studyfrog-calendar`, `crates/studyfrog-store`, `crates/studyfrog-ui`)

- Homework items with title, course, start and due date and time, estimate, priority, status, checklist and notes. They are stored in SQLite and mirrored to `.studyfrog/homework.toml` on every change (STORY-0165).
- Lead-time reminders at 24 hours, 2 hours and 15 minutes before a deadline. Each is shown once and remembered across restarts. Calendar and homework reminders share one engine (STORY-0166).
- A Homework screen in the Plan group with All, Active, Due today, Overdue and Completed tabs, a countdown and urgency on each item, checklist ticks, and a dialog to create and edit items. Deadlines appear on the calendar grids and in the day view (STORY-0167).

### Universal Import and less crowded screens (EPIC-057, `crates/studyfrog-ui`, `crates/studyfrog-service`)

- One Import entry point in the items toolbar, the stack menu and the More menu. The Import tab says which importer a chosen path looks like and where the items go: the vault root, an existing stack, or a new stack (STORY-0168).
- PDFs import as one card per page with text. Markdown card decks import as one card per `## ` heading.
- Ctrl+F (and `/` when nothing has the keyboard) focuses search on the library and in the PDF viewer (STORY-0188).
- Tooltips with shortcuts on every icon button; every text button explains itself on hover, and a source check fails when a new one does not (STORY-0169, STORY-0189).
- Low-frequency buttons moved into overflow menus; roomier Comfortable and Spacious densities; toasts at the bottom right (STORY-0170).

### FrogScript wiki and cookbook (EPIC-058, `crates/studyfrog-script`, `crates/studyfrog-ui`)

- A searchable FrogScript reference beside the editor: syntax, operators, control flow, built-in functions, vault objects and lifecycle events. An example can be added to the script (STORY-0171).
- A cookbook of ten recipes; each can be inserted into the editor, and the read-only ones run in a new console tab (STORY-0172).

### Memory Mode (EPIC-059, `crates/studyfrog-rehearsal`, `crates/studyfrog-ui`)

- Face-down card pairs of front and back, turned two at a time, with a seeded shuffle, move and streak counters, and keyboard play (STORY-0173, STORY-0174).
- Stopwatch or a 60, 90 or 120 second countdown, a victory summary, and the best time of each stack kept across restarts (STORY-0175).

### Cognitive profile (EPIC-060, `crates/studyfrog-store`, `crates/studyfrog-report`, `crates/studyfrog-srs`)

- Each timed rating stores latency, dwell and text size in SQLite (STORY-0176).
- A calibrated starting difficulty for new items, blended from the user's own first answers. On by default; a switch in Settings, Scheduling turns it off (STORY-0177).
- A Cognitive profile tab on the Reports screen: latency against word length, weekly passing share, learning velocity, a metacognitive calibration widget and a consolidation chart (STORY-0177, STORY-0178).

### Case studies and math drills (EPIC-061, EPIC-062, `crates/studyfrog-rehearsal`, `crates/studyfrog-agent`, `crates/studyfrog-ui`)

- Scenario-based case studies: a vignette, a decision with options and expected choices, and a rubric in front matter. A choice and a justification are scored against the rubric (40 % choice, 60 % rubric) (STORY-0179, STORY-0180, STORY-0181).
- Psychometric formulas (standard error of measurement, confidence intervals, standard scores, Spearman-Brown, z-values) with a seeded exercise generator and a step-by-step check that names the first wrong row and its likely mistake (STORY-0182, STORY-0183).
- A math drill with a keypad, optional working rows and a step reveal (STORY-0184).

### Altklausur mock exams (EPIC-063, `crates/studyfrog-rehearsal`, `crates/studyfrog-agent`, `crates/studyfrog-ui`)

- Mock exams with weighted sections, flags, an unanswered-question review and a strict timer with 10-minute and 2-minute alerts (STORY-0185).
- "Mock exam" on the rehearse setup makes ten questions from a stack's notes, or from a lecture PDF in the vault. The questions are balanced 40/30/30 and saved into the stack with reference answers (STORY-0186).
- Finishing grades each answer against its rubric and shows the grade on the German scale 1.0 to 5.0 (pass at 50 %), the points by topic and a review of every question (STORY-0187).

### Learn modes (EPIC-066, `crates/studyfrog-rehearsal`, `crates/studyfrog-store`, `crates/studyfrog-ui`)

- A learn mode is a graph of start, present, recall, rate, schedule, log, branch and stop steps. It is drawn on a canvas in the learn-mode editor, checked before it is saved, and run as practice only (STORY-0198).
- Custom runs have their own report and are kept apart from the built-in history.

### Control protocol (EPIC-064, `crates/studyfrog-ui`, `crates/studyfrog-service`)

- The command protocol covers homework, calendar, reminders, search focus, tooltips, import destinations and PDF find, so the app can be driven and checked from a script (STORY-0190, STORY-0191, STORY-0192, STORY-0194).

### Benchmark tooling (EPIC-067, `scripts/`)

- `scripts/bench-reference.sh`, `scripts/bench-machine.sh`, `scripts/bench-spread.sh` and `scripts/bench-compare.sh` record reference runs with the machine's identity and compare two runs with a bootstrap rule. The tooling is tested here; the baselines are not measured yet (see Known limits).

## Verification

- `scripts/check.sh` passes on the release commit: formatting, repo-lint, strict clippy (`-D warnings`), the full workspace test suite (2232 tests, no failures), the 250-line file limit, layering, the no-inherent-impl scan, issue front matter and the board.
- Packaging: `scripts/package-appimage.sh` builds the release binary and the tarball, each tarball with a SHA-256 file. The AppImage could not be packed on the build machine (a system tool was missing); the tarball is verified by checksum and the packaged binary starts headless. See `docs/RELEASE_AUDIT_1.0.0.md`.

## Known limits

- **Not tagged or published.** The `v1.0.0` tag, the GitHub release and the uploaded artifacts wait for the owner.
- **Owner's-machine checks not done.** The reference benchmark runs and the measured baselines (EPIC-067, STORY-0201, STORY-0203) need the owner's machine. Until they are run, the performance budgets in `docs/contributing/BENCHMARKING.md` are provisional.
- **Window checks still open.** Several stories are built and tested headless but were not yet clicked through in a running window: STORY-0150, STORY-0151 and STORY-0152 (remaining criteria in ScriptStudio and the console), STORY-0177, STORY-0192 and STORY-0194 (focus, calendar movement and reminder peek), STORY-0186 and STORY-0187 (the mock exam view), and STORY-0198 (a custom run and its report).
- **Model grading is not measured.** The local-model grading of case justifications (STORY-0180, STORY-0200), mock exam answers (STORY-0187) and the generated exam questions (STORY-0186) was tested with a fake model. No real language model was run for this release, and small local models are unmeasured. Without a model, the keyword check grades, and the result says so.
- **Some milestone work is still in review.** Of the 23 stories of the v1.0.0 roadmap (STORY-0165 to STORY-0187), 7 are done and 16 are in review. EPIC-058 (FrogScript wiki) and EPIC-065 are in the backlog as epics, although their stories are in review. The board shows the status of each item.
- **No Altklausur lab screen.** Mock exams are started from the rehearse setup; the dedicated lab screen is not built.
- **Heavy release tests not run.** The model-dependent release tests (`cargo test --release -p studyfrog-candle`) need an installed language model and were not run for this release.
- **AppImage not yet packed or checked.** The AppImage for this release has not been built: packing needs `desktop-file-validate`, which the build machine lacks. It has also not been started on a real desktop session with a vault open (RELEASING.md step 6). The tarball is built and verified by checksum.
- Linux on x86_64 only. It needs OpenGL and X11 or Wayland from the system.

## Install

StudyFrog is for Linux on x86_64. It requires OpenGL, and X11 or Wayland, from your system.

### AppImage

The AppImage is added to the release once it is packed (see Known limits).

```sh
sha256sum -c StudyFrog-1.0.0-x86_64.AppImage.sha256
chmod +x StudyFrog-1.0.0-x86_64.AppImage
./StudyFrog-1.0.0-x86_64.AppImage
```

### Tarball

```sh
sha256sum -c StudyFrog-1.0.0-x86_64.tar.gz.sha256
tar -xzf StudyFrog-1.0.0-x86_64.tar.gz
cd StudyFrog-1.0.0
./bin/studyfrog
```

## Upgrading from 0.9.0

Back up your vault before the first start. Opening a vault with 1.0.0 applies three new database migrations (schema steps 29, 30 and 31: homework items, cognitive telemetry, and custom learn-mode runs). They add tables and do not change existing notes, cards or review history. The migration tests pass; no upgrade of a real vault was run for this release.
