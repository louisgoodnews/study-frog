# Changelog

All notable changes to StudyFrog are recorded here, newest first.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). The project is pre-release, so
everything is under `Unreleased` until the first tagged version. Each entry names the work item
(`TASK-xxxx`) it belongs to where one exists.

## [Unreleased]

### Fixed
- Teacher and subject names may contain commas ("Prof. Dr. Künkelmann, Harry"): a stack's `subjects` list is written as a
  quoted list when needed and read back whole, and the Settings screen refuses a name that cannot be stored before saving,
  keeps what was typed and says why next to the field (BUG-0001).

### Added
- Settings are in tabs by domain (Study and scheduling, Rehearsal, Answers and AI, Data, Subjects and teachers); the last tab is
  remembered in `app.toml` as `settings_tab`, and the Left and Right keys move between tabs. Tabs for General, Appearance and
  Notifications appear when they have settings (TASK-0142).
- Specification of item statuses (`docs/specs/status.md`): five statuses, where they are stored, who may change them, what they do to
  the schedule, and how a stack rolls up its items (TASK-0143).
- Priority order for 0.3.0 in four waves in `docs/roadmap/M10-usability-and-insight.md`; wave 1 is `ready` on the board (BUG-0001,
  TASK-0128, TASK-0135, TASK-0142, TASK-0143).
- Research and a proposal for combining familiarity with FSRS: a relative factor on the FSRS interval that leaves the stored
  state untouched, a reproducible simulation (`docs/research/familiarity-fsrs-sim.py`) and a replay that tells each user whether it
  helps; it is on from the start (TASK-0183, TASK-0184).
- Planning for 0.3.0 (phase M10, EPIC-023): notification management, the vault in the bar, a library with an SQL-like
  search, a stack overview with sub-views and a separate stack detail view, a familiarity value used in reports and scheduling, tabbed
  settings, item statuses, calendar entries for the next reviews that can be moved by drag and drop, import and export
  with Anki and Obsidian importers, a built-in wiki with FAQ and tutorials, a diary and rehearsal modes for countdown and count up, recorded as STORY-0055 to
  STORY-0070, BUG-0001 and TASK-0126 to TASK-0184; the issue checker accepts phase `m10`.
- Releases carry a `.tar.gz` (binary, desktop entry, icon, licences, README, changelog, release notes) and its `.sha256`
  next to the AppImage; the packaging script writes them and the release workflow uploads them (TASK-0125).

## [0.2.0] - 2026-10-31

### Added
- Links inside a PDF: outlined on the page, page links followed with a click, web and mail addresses opened only after
  the user presses Open on a question that shows the address; launch, script and other actions are never run (TASK-0123).
- Documentation, tests and benchmarks for PDF support: `docs/specs/pdf.md`, headless view tests with the accessibility
  audit, a 500-page benchmark with budgets, and draft release notes `docs/RELEASE_NOTES_0.2.0.md` (TASK-0122).
- PDFs in the library: the stack page lists its PDFs with page counts, 'Add PDF ...' (also in the Create dialog) copies a
  file into the stack under a safe name, renaming a PDF moves its notes file and rewrites every note, card and link that
  names it, `[text](file.pdf#page=3)` and `![](file.pdf#page=3)` open the viewer at that page, and the home page lists the
  PDFs that changed most recently (TASK-0121).
- Cards and questions from PDFs: a highlighted passage or the selected words become a card or a question with a link to
  the page, shown as a 'Source' link while rehearsing; the assistant can read chosen PDF pages (shown first, sent only
  after Send) and propose cards and questions that name the page they come from (TASK-0120).
- Pictures from PDFs: save the pictures of a page or a dragged rectangle of a page (figures, formulas) into the vault's
  assets folder under the usual picture rules, and put them at the end of a note made from the page (TASK-0119).
- Notes from PDF text: the text of pages or of a selection becomes a Markdown note in a stack, with a `source` key that
  leads back to the page; the item page offers 'Open source' and the viewer lists the notes made from the file
  (TASK-0117). Pages that are only pictures are flagged in the index and the viewer, and notes made from them say so
  (TASK-0118). `docs/specs/pdf.md` describes the formats and limits.
- Highlights and notes on PDFs: choose a colour, highlight the selected words or place a note, edit and delete them in a
  list, jump to each; stored in plain text in `name.pdf.notes.toml` next to the PDF, searchable, included in backups, and
  followed to the right page if the PDF is replaced by a revision (TASK-0114).
- Rearranging PDFs: turn, move and drop pages in a thumbnail grid with undo, append other PDFs, split by page ranges and
  save as a new file with new title, author and subject and, if wanted, the highlights and notes written into the copy
  as standard annotations; the original is never changed (TASK-0115, TASK-0116).
- PDF files in the vault are indexed (title, page count and the text of every page, extracted once per change) and the
  global search finds them by title and by text, showing the page of the hit (TASK-0111).
- The PDF viewer: one page at a time with zoom (fit width, fit page, percent), page jump, outline, page thumbnails,
  search inside the file with highlighted hits, word selection by dragging and copy, and the page's text for screen
  readers; pages are drawn on the worker thread and cached (TASK-0112, TASK-0113).
- The `studyfrog-pdf` crate (TASK-0110, TASK-0113): reads page sizes, outline, properties, words with boxes, text,
  Markdown and pictures; draws pages and regions; rotates, drops, reorders, merges and splits pages; writes
  highlights and notes into a copy. Size, page, pixel and time limits and crash containment keep hostile files from
  taking the app down.
- PDF library spike (TASK-0109): measured findings in `docs/research/pdf-library-spike.md` and a proposed ADR 0010
  (hayro to draw pages, pdf_oxide to read and edit, both behind a `studyfrog-pdf` crate); ADR 0010 is accepted.
- Planning for PDF support: EPIC-022 with STORY-0054 and TASK-0109 to TASK-0122 (view, annotate, rearrange and extract
  PDF files), a new phase M9 in the roadmap and the `m9` label.

## [0.1.0] - 2026-10-31

### Added
- The Assistant screen: connect a local model (Ollama and similar) or a remote one (OpenAI-compatible, Anthropic),
  let it read chosen stacks and propose changes like an LLM wiki editor (new notes, cards, questions, edits, an
  index) that you accept one by one, and be examined by it with the verdicts recorded as reviews only when you say
  so. A remote model is refused until you allow it; keys live in a private file in your config folder; calls are
  logged without their content (STORY-0049, ADR 0009).
- Backups and restores (Settings, and the welcome page): one plain `.tar` with the vault's files, a consistent copy of
  the database (review history, runs, the board and the calendar), and optionally the settings and API keys. A
  restore goes into a new or empty folder, never overwrites anything, and keeps the old settings next to the new
  ones (STORY-0050).
- User profiles: each has its own settings, recent vaults and API keys; switch, create and delete them from the
  profile menu (or the welcome page). The vault path is no longer shown in the profile menu (STORY-0051).
- Pictures in notes, cards, questions and answers: a Picture button in the Markdown toolbar copies a png, jpg or gif
  into the vault's `assets/` folder and inserts `![alt](assets/name.png)`; pictures show in the editor preview, the
  item page and while rehearsing, and only pictures inside the vault are loaded (STORY-0044).
- A Performance tab on the reports screen: reviews, items reviewed, good-or-easy share, answer times, rating bars
  and a 30-day chart for the whole vault, a stack, a subject or one item, over all time, 30 or 7 days, drilling
  down by clicking (STORY-0048).
- The item page: title, a Markdown editor for every text area (toolbar, Write, Split or Preview), and a side column
  with the kind, stack, subjects, priority, difficulty, tags, dates, the links to the item and the items that link
  to it (STORY-0042, STORY-0043). The Create dialog uses the same editor, wide for notes.
- Priority (lowest to highest) and difficulty for stacks, notes, cards and questions, inherited from the stack;
  priority orders spaced runs and breaks ties in the other orders, and the owner's difficulty stands in for the
  memory difficulty of new items (STORY-0053).
- Teachers and subjects (`subjects.toml`, edited in Settings); stacks name subjects and what is inside takes them
  (STORY-0045).
- A date picker (a month grid under the date field) in the Create dialog and the calendar form; a stack's due date
  can be set or changed from its Library page (STORY-0046, STORY-0052).
- Stacks whose due date has passed are left out of the suggested run and the home page's counts; choosing such a
  stack by hand in the run setup still works (STORY-0052).
- Cards made by an import or by the Library card form also get a board card (the first 25 of an import, then one
  summary card); board cards are draggable by their whole face and columns light up as drop targets (STORY-0047).
- ADR 0009 (accepted): how an AI agent will work the vault, where its keys and logs live, and that it only
  proposes changes (STORY-0049). New backlog stories for backups, profiles, priority and difficulty
  (STORY-0050, 0051, 0053).
- The end-of-run review is a summary with tiles and bars and a scrollable card for every item; prompts and answers
  are drawn as Markdown while rehearsing (STORY-0040).
- The vault's folder name and path no longer appear in the top bar, the home page, the status line or an "Opened"
  notice; the status line shows the size of the vault instead (STORY-0041).
- Calendar entries are now study sessions, tasks, appointments, exams or deadlines, each with up to five reminders
  that show once as a toast and a notification while the app is open; the month grid fills the window with
  coloured entries and a "Coming up" list (TASK-0105).
- A stack can be given a due date and reminders in the Create dialog, which puts a deadline on the calendar; every
  item made in the dialog also gets a card in the first board column (TASK-0106).
- The run setup is three steps: What, How, Review (TASK-0107). The home page lists the items due now and the next
  seven days and fills the window (TASK-0108).
- `docs/specs/UX_EVIDENCE.md`: the usability research behind these changes, with how strong each finding is
  (TASK-0104).
- A home page (greeting, items ready today, streak, stacks waiting with a Rehearse button, planned sessions,
  recent items and runs) that a vault opens on; a shared theme (palette, text sizes, spacing) for every screen;
  stacks as cards and items as rows in the library; a card layout, progress bar and coloured rating buttons in
  the rehearsal screen (TASK-0100 to 0103).
- A Create dialog (modal) for stacks, cards, notes and questions with a parent-stack picker, tags and "Create
  another"; stacks can sit inside other stacks (shown as a tree in the library, and run setup includes the stacks
  inside a ticked one); first launch shows a welcome page and a clickable in-app notification that opens the
  vault dialog, instead of a path field (TASK-0096 to 0099).
- A top bar (sidebar toggle, vault switcher, brand, global search, Create menu, notifications, help, settings,
  profile) and a foldable sidebar for the screens; global search over item titles; a notification list behind
  the bell (TASK-0093, 0094, 0095).
- Extensions: `studyfrog-ext` (importers and card types as plain data, a registry), an example extension
  (tab-separated importer, cloze card type) that changes no core crate, a card-type chooser and an "Import from
  a file" panel in the library, `docs/EXTENDING.md` and ADR 0008 (TASK-0063, 0064, 0065).
- Packaging: `scripts/package-appimage.sh`, a desktop entry and icon, and a CI workflow that runs the gate and
  builds the AppImage (TASK-0067). The 0.1.0 AppImage builds and starts locally; the workflow has not run yet.
- Choose the vault folder with the native dialog (through the desktop portal; the app binary supplies it, the
  UI crate does not link the dialog library) (TASK-0092).
- Shuffling is its own switch: any order, spaced repetition included, can be shuffled without changing which
  items are chosen or whether the schedule is updated (schema v7; runs stored as `shuffled` become sequential
  with the switch on) (TASK-0091).
- Open flags closed: answer runtimes are calibrated on a labelled set (word matching gets a pivot of 0.44;
  the language model keeps 0.7); typed-answer scores are stored with their run (schema v6) and shown in the
  review; `answer_engine = "words"` and a Settings choice force word matching; the user's easy/difficult mark
  suggests a rating; the fast/slow/minimum-words thresholds are configurable in `app.toml` (TASK-0086 to 0090).
- Reports: retention by time since the previous review, time to answer (percentiles, histogram, by the
  difficulty mark), and a 30-day review-load forecast with a backtest on real or simulated history; a Reports
  screen; each report computes in under 10 ms on 100,000 reviews (TASK-0059, 0060, 0061, 0062).
- Achievements: rules as data (built-in set or the vault's `achievements.toml`), statistics from the review
  history including streaks, unlocks remembered once (schema v5), an Achievements screen with progress and
  an announcement when something unlocks (TASK-0056, 0057, 0058).
- Reading speed can be measured on the Settings screen (read a lorem ipsum text, press done) and is kept in
  `app.toml`; rehearsal uses it as the baseline. Reading much slower than expected now suggests a harder
  rating (Good to Hard, Easy to Good) (TASK-0084, 0085).
- Rehearsal: mark an item Easy, Medium or Difficult while it is on screen (buttons or E/M/D); reading speed
  against a 238 words-per-minute baseline suggests a rating; the post-run review shows previous rating,
  own mark and rating and the suggestion, and can apply it, rebuilding the item's schedule (TASK-0081,
  0082, 0083).
- Semantic answer comparison: a candle MiniLM runtime (`studyfrog-candle`, linked by the app only) used
  automatically when `all-MiniLM-L6-v2` is installed under the app data directory, with fallback to word
  matching and the reason shown; the result line says which engine compared the answer. Measured: p95
  about 75 ms per comparison, about 85 MB of memory (TASK-0050, 0051).
- Answer comparison: `SlmRuntime` trait, scoring over embeddings (best of reference and accepted answers,
  rubric points, verdict bands, suggested rating), an in-house offline lexical runtime, property tests and a
  benchmark; typing an answer in rehearsal now shows the match, the verdict and a suggested rating
  (TASK-0049, 0052, 0053, 0054, 0055).
- Calendar screen: month grid with recurring sessions, day panel (edit, delete, skip a day, start a
  rehearsal for the session's stack) and a session form (TASK-0045).
- Board screen: columns with WIP counts and over-limit marking, drag and drop plus push/pull/up/down buttons,
  card editor with stack link, optional forcing past WIP limits (TASK-0048).
- Planning foundations: `studyfrog-calendar` (sessions, recurrence checked against a day-by-day oracle) and
  `studyfrog-board` (columns, WIP limits, strict/forced moves, push/pull, commands; property-tested); schema v4
  (sessions, board); service use cases for both; `core::civil` (dates, weekdays, month grid, local time); a
  local time-zone provider so daily limits and run previews use the local day (TASK-0044, 0046, 0047, 0078,
  0079, 0080).
- Link graph screen: the wikilink graph as a force-directed picture with pan, zoom, hover, click-to-open, a
  re-run button, an accessible item list, and live refresh that keeps node positions. New pure crate
  `studyfrog-graph` (SoA layout, uniform-grid repulsion, deterministic, zero allocation per step): 5k nodes /
  10k edges step in 4.0 ms; verified allocation-free with dhat. Items now show when they are next due
  (TASK-0042, TASK-0043). EPIC-009 and phase M4 complete.
- Editor: open any item from the browser, edit title, tags and text areas with a live Markdown preview (headings,
  emphasis, lists, task items, quotes, code, wikilinks and URL links), undo/redo, save with conflict detection
  (a file changed elsewhere is never overwritten), revert/reload, unsaved-changes guards, clickable wikilinks.
  `studyfrog-md::blocks` (preview model, pulldown-cmark), `studyfrog-service::document`, `resolve_title`
  (TASK-0040, TASK-0041).
- Rehearsal screen: keyboard-first (Space, 1-4, S, P, Esc), pause that hides the question, flip and type-answer
  modes, look-ahead content loading, end-of-run summary; every step saved through the service (spaced runs update
  the schedule, practice-only runs do not). Service `start_run`, `load_item_content`, `StepContext`; crashed runs
  are recovered when a vault opens. 15 tests (TASK-0039). EPIC-008 (UI shell) complete.
- Run setup screen: presets (spaced, everything in order, shuffled, hardest first, not seen for longest),
  all orders, filters (kind, stack, tags, state, difficulty), limit, requeue, and an explicit "update my
  schedule" switch; live count of matching items from the index. 6 tests (TASK-0073).
- Stack browser: choose a stack or the inbox, filter its items by title, create stacks and cards from the UI.
  Service use cases `create_stack`/`create_card` (never overwrite, validated, reindexed at once), in-house ULID
  composition with OS randomness, `list_items`; 14 new tests incl. a real-vault build-a-stack flow
  (TASK-0038).
- First desktop app (EPIC-008 so far): eframe/egui window with a vault chooser (remembers the last vault),
  library overview (stacks, counts, inbox, problems), navigation and settings view; pure
  `update(State, Msg) -> (State, Vec<Effect>)` with a worker-thread executor and 1 s change polling;
  21 tests incl. headless egui view tests and an accessibility audit; `--smoke-frames N` for CI
  (TASK-0036, TASK-0037).
- `studyfrog-service::session` (open a vault, index it, poll it), `studyfrog-store::overview` (stacks, counts,
  problems), per-user `app.toml` (`studyfrog-config`, `studyfrog-store::app_config`), `poll::prime`.
- `studyfrog-config` (ADR 0007, Proposed): pure parsing and rendering of `studyfrog.toml` (vault defaults:
  scheduler, retention, interval cap, FSRS weights) and `_stack.toml` (scheduler, retention, daily limits);
  unknown keys are errors with line numbers (TASK-0075).
- Settings pipeline: `_stack.toml` is indexed (`stack_identity`, `stack_config`, view `stack_setting`), loaded into
  the library, and included in polling; schema v3 (TASK-0075).
- SM-2 in the service: each item is scheduled by its stack's scheduler (default from `studyfrog.toml`),
  stack retention overrides apply to FSRS, switching a stack keeps items scheduled (TASK-0076).
- Per-stack daily new/review limits for spaced runs, counted from the review history with an injected day window;
  practice-only runs do not count (TASK-0077).
- Run filters are persisted with the run and round-trip exactly (TASK-0074). EPIC-018 complete.
- `studyfrog-service` (application layer, ADR 0006 Proposed): `apply_rating` (pure), `record_step`,
  `begin_run`, `recover_runs`, `new_run`/`candidates`; end-to-end tests over a real vault and database
  (spaced run updates the schedule; practice-only run leaves it untouched; due items return most-overdue
  first; crash recovery) (TASK-0033).
- `studyfrog-store`: schema v2 (run plan columns, in-place upgrade tested), run and event persistence
  with crash recovery, review and FSRS state storage, item positions (TASK-0072, TASK-0033).
- `studyfrog-core`: event name/payload encode and decode. Phase M3 complete.
- Queue builders in `studyfrog-rehearsal`: filters (kind, stack, tags case-insensitive incl. non-ASCII,
  state, difficulty), orders (spaced with daily budgets, sequential, seeded shuffle, hardest/easiest,
  stalest, lowest predicted recall), length cap; in-house SplitMix64 shuffle; 13 tests incl. an
  independent filter oracle; benchmark: engine step 23 ns, 10k-candidate queue 82 us to 3.9 ms
  (TASK-0034, 0069, 0070, 0035).
- Run plans in `studyfrog-core`: orders (due-first, sequential, shuffled, hardest/easiest, stalest,
  lowest predicted recall), filters, schedule effect (update vs practice-only), requeue-on-Again, limit
  (TASK-0068). Rehearsal is not only spaced repetition.
- `studyfrog-rehearsal`: pure run state machine (`step`), injected clock, latency excluding paused
  time, requeue with cap; 9 tests incl. an invariant property (TASK-0030, 0031, 0032, 0071).
- `studyfrog-store`: vault walker, in-house content hash, stack ownership by folder, non-fatal scanner
  with duplicate-id detection, `Library` builder (inbox stack for orphans), incremental reindex
  (stat-only for unchanged files, rename-safe, duplicate-healing), polling change detection with
  settle window; 17 tests and a 10k-note benchmark (cold index 0.57 s). EPIC-006 and phase M2
  complete (TASK-0027..0029).
- `studyfrog-store`: SQLite schema v1 (derived index + authoritative history), transactional
  migration runner, WAL/foreign-key pragmas, 6 tests (TASK-0025, TASK-0026).
- `studyfrog-md`: in-house ULID and RFC 3339 codecs, front-matter fence/key-block splitter, typed
  parser and renderer for stack/note/card/question with verbatim unknown keys, `???` splitter,
  wikilink extraction, duplicate-id check; 25 tests incl. 1000-case round-trip property
  (TASK-0020..0024). EPIC-005 complete.
- `studyfrog-srs`: SM-2 (`sm2_next`, ease/interval rules from the original), `Sm2State` in core, and a
  criterion benchmark (100k cards: FSRS 2.9 ms, SM-2 0.72 ms); FSRS budget tightened to 10 ms
  (TASK-0018, TASK-0019). EPIC-004 and phase M1 complete.
- `studyfrog-srs`: FSRS-6 (`initial_state`, `next_state`, `retrievability`, intervals, Anki-style
  deterministic fuzz); `MemState` in core; golden test reproducing the py-fsrs reference intervals and
  7 property tests (TASK-0016, TASK-0017).
- `studyfrog-core`: `Run`, `RunMode`, `RunState`, `RunEvent`, `EventKind`, `Latency` and persisted-name
  functions, 2 tests (TASK-0015). EPIC-003 (core domain types) complete.
- `studyfrog-core`: `Rating` (persisted values 1-4), `Review`, `rating_value`/`rating_from_value`,
  3 tests (TASK-0014).
- `studyfrog-core`: `Meta`, `Stack`, `Note`, `Card`, `Question`, `UnixMs` and `Library` (one arena per
  kind, no `Item` enum), plus a test (TASK-0013).
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
- The Create button opens the dialog (the dropdown with study session, board card and import entries is gone;
  those are made on their own screens). The inline "new stack" form in the library is replaced by "New stack…".
- The screen buttons moved from the top row to a sidebar; "Close vault" moved into the profile menu.
- The lowest-recall queue order is 7 times faster (recall computed once per item) (TASK-0066).
- The application binary is called `studyfrog`.
- Dev builds use line-table debug info only, so the many test binaries link much faster.
- `md`'s RFC 3339 codec shares `core::civil` instead of carrying its own date arithmetic.
- Daily limits and run creation use the local day (was the UTC day).
- Queues are built from the index (`studyfrog_store::overview::load_indexed_items`) instead of a parsed
  `Library`; `new_run` no longer takes a library.
- Stack settings (`daily_*`, `srs`) are no longer `_stack.md` front-matter; they moved to `_stack.toml`
  (ADR 0007). `StackDoc` now carries only identity, description and body; core `Stack` has `settings`.
- ADR 0005 amendment accepted: no `gray_matter`; in-house fence splitter plus `serde-saphyr`
  (TASK-0021 retitled). New design principle P10: build instead of buy when we can do better.
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
- Graph layout never came to rest because the repulsion jumped to zero at its cutoff (nodes flickered in and out of
  range); a smooth window fixes it. Found by measuring energy, not by a failing test.
- Board/issue bookkeeping: TASK-0016, STORY-0007 and EPIC-004 were still `backlog` after M1. `repo-lint`
  now rejects a parent whose children are all done but is not, a done parent with open children, and a
  done item with unchecked acceptance criteria.
- `scripts/linecount.sh` no longer miscounts the `wc` total line on non-English locales.
