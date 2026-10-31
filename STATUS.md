# Status

**Phase:** M0 to M9 complete; released 0.1.0, PDF support (0.2.0) waits for the version bump. **Date:** 2026-10-31. **Branch:** `main`.
Live work: [BOARD.md](BOARD.md). History: [CHANGELOG.md](CHANGELOG.md).

## What exists
- A Cargo workspace of 20 crates under `crates/` plus an example extension under `examples/` (layering enforced by `scripts/check-deps.sh`, ADR 0001):
  `core`, `config`, `md`, `store`, `srs`, `rehearsal`, `calendar`, `board`, `graph`, `slm`, `candle`,
  `report`, `achieve`, `ext`, `agent`, `net`, `pdf`, `service`, `ui`, `app`. No inherent `impl` blocks; files are at most 250 lines.
- The app (`cargo run -p studyfrog-app`): vault of Markdown notes, cards and questions with links and a graph;
  FSRS-6 and SM-2 scheduling per stack; rehearsal runs in any order with latency capture, difficulty marks, a
  reading-speed baseline and a review after each run; typed answers compared offline (language model or word
  matching); calendar and kanban board; achievements; retention, answer-time and forecast reports.
- Guardrails: `scripts/check.sh` (fmt, clippy, tests, line count, layering, no-OOP, issue and board checks).

## Latest
- **Next:** the owner bumps the version (0.2.0); draft notes are in `docs/RELEASE_NOTES_0.2.0.md`. EPIC-022 (PDF
  support, phase M9) is done: viewer, index and search, highlights and notes, rearranging, notes, cards and pictures from
  PDFs, assistant reading, library; OCR is deferred (docs/specs/pdf.md).
- **Not verified by hand in a window:** the PDF viewer, thumbnails, highlights and the arrange grid were tested headless
  only (see the release notes for what else).
- EPIC-021 (knowledge workspace) is done: item page and Markdown editor, pictures, priority and difficulty,
  subjects and teachers, the performance explorer, backups, profiles and the AI assistant (docs/specs/agent.md).
- **Not verified by hand in a window:** the item page, the Markdown toolbar, the date picker, the assistant screen,
  the backup and profile menus and the new board cards were tested headless and by logic only. The Home, Calendar
  and rehearsal screens were looked at in a real window earlier.
- **Never run against a real provider:** the three wire formats follow the providers' documentation and are tested
  against a local fake server only.
- Home page, theme, library cards and rehearsal card layout are in (STORY-0038). The home summary reloads when the
  home or library screen opens, not on every vault change.

## Decisions
- **Accepted by the owner:** ADR 0006 (service layer); run defaults (spaced runs update the schedule, the
  presets that go through everything and the drill presets are practice-only), with shuffling as a separate
  switch that works for every order; a native folder dialog for choosing the vault.
- **ADR 0008 accepted by the owner:** extensions are records of function pointers in a registry, compiled in.
- **ADR 0007 accepted by the owner:** settings live in TOML next to the Markdown content (`studyfrog.toml`,
  `_stack.toml`, `achievements.toml`, `app.toml`), keyed by folder and id.

## Known limits
- **Answer comparison** measures similarity, not truth: a wrong answer that differs from the reference by one
  decisive word usually scores as correct (10 of 11 in the labelled set). The language model is optional and
  must be placed in `~/.local/share/studyfrog/models/all-MiniLM-L6-v2/`.
- **Forecast** error (weekly totals off by 9 to 11 %) was measured on a simulated history only; the Reports
  screen says which history it used.
- **Marks** (easy/medium/difficult) are stored and used in reports and rating suggestions, but never change the
  schedule by themselves.
- **UI:** the folder dialog goes through the desktop portal and has not been tried on a real desktop session;
  wikilinks resolve by title only; IME and real screen-reader behaviour are unverified (ADR 0002).
- **Settings:** the renderer does not preserve comments (a settings UI should use `toml_edit`); `studyfrog.toml`
  is read on demand, with no hot reload; after a scheduler switch `Review.elapsed_days` restarts for the new
  algorithm.
- **Performance:** lowest-recall queue building is 3.9 ms for 10k items (a `powf` per comparison).
- **Extensions** are compiled in (no plugin loading, no sandbox); an extension cannot keep settings or state.
- Macro-generated impl detection in `repo-lint` is heuristic. Citations marked `[u]` in `docs/research/` are
  unverified from memory. No git remote is configured.

## Next
- **The AppImage builds and starts** (`scripts/package-appimage.sh`: 9.5 MB, SHA-256 `40721ef0...6825fd`, stripped
  binary, no libraries beyond libc; OpenGL, X11, Wayland and xkbcommon come from the host). The CI workflow
  (`.github/workflows/ci.yml`) passes `actionlint` but has never run because there is no git remote: that is the
  one unmet M8 exit criterion ("builds from CI").
- Then cut the release by the steps in [RELEASING](docs/contributing/RELEASING.md); the notes are drafted in
  [RELEASE_NOTES_0.1.0](docs/RELEASE_NOTES_0.1.0.md). The version is already 0.1.0, so there is no bump.

## Using the app now
`cargo run -p studyfrog-app`: pick a vault folder, create a stack and cards, set up a run (any order or filter),
rehearse from the keyboard. `--smoke-frames N` draws N frames and exits.

## M4 summary
- Link graph (pan/zoom/click, 5k nodes in ~4 ms per frame, zero allocation) and a Markdown editor with live preview and conflict-safe saving. Window, navigation, vault chooser with remembered vault, library overview, stack browser (filter, create stack
  and card), settings view, run setup with presets/filters/live count, rehearsal screen and summary.
- Pure `update(State, Msg) -> (State, Vec<Effect>)`; effects run on a worker thread; vault use cases live in
  `studyfrog-service`. About 70 UI tests, all headless (egui_kittest), plus an accessibility audit.

## M3 summary
- `studyfrog-core`: `RunPlan` (order, filter, effect, seed, limit, requeue) and persisted names.
- `studyfrog-rehearsal`: pure `step` state machine (validate-then-mutate), injected clock, latency without
  paused time, requeue with cap; queue builders for all orders and filters (seeded in-house shuffle).
  Engine step 23 ns; 10k-candidate queues 82 us to 3.9 ms.
- `studyfrog-store`: schema v2, run/event persistence, crash recovery, reviews and FSRS state.
- `studyfrog-service`: end-to-end tests over a real vault and database.

## M2 summary
- `studyfrog-md`: in-house ULID and RFC 3339 codecs, fence/key-block splitter, typed parse and render for
  all four kinds, unknown keys preserved verbatim, `???` splitter, wikilinks, duplicate-id check.
  1000-case round-trip property.
- `studyfrog-store`: SQLite schema v1 + migrations, vault scanner (non-fatal errors, duplicate ids),
  library builder (inbox stack), incremental reindex (rename-safe), polling change detection.
  10k notes: scan+parse 282 ms, cold index 571 ms, warm reindex 43 ms (budget 3 s).
- Decisions: polling instead of a watcher crate; canonical key order with verbatim unknown keys;
  orphans go to an implicit Inbox stack (uid 0).

## M1 summary
- `studyfrog-core`: `Uid`, `Idx<T>`, `Arena<T>`, `Meta`, `Stack`, `Note`, `Card`, `Question`,
  `Library` (one arena per kind), `Rating`, `Review`, `Run`, `RunEvent`, `Latency`, `MemState`,
  `Sm2State`, `UnixMs`.
- `studyfrog-srs`: FSRS-6 (golden test reproduces py-fsrs intervals; 7 properties), SM-2 (hand-computed
  vectors + properties), deterministic fuzz. Benchmarks: FSRS 2.9 ms / 100k cards, SM-2 0.72 ms.
