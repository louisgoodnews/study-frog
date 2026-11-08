# Status

**Phase:** M0 to M11 complete; M12 partly built; released 0.5.0 (part of the planned scope). **Date:** 2026-11-08. **Branch:** `main`.
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
- **EPIC-036 (UI aesthetics) in progress** on `feature/complete-m12`: STORY-0113 done (OKLCH palettes in `theme_tokens.rs`, in-house Oklab/OKLCH/APCA in `color_science.rs`, APCA gate `tests/appearance_apca.rs` over all 24 palettes; spec `docs/specs/ui-colour-and-type.md`). STORY-0114 in review (type scale, 1.5 line height, a 70-character reading measure of about 450 pt rather than the 620-680 px the story names, tabular timers). Checked by tests and screenshots, not in an interactive window.
- **EPIC-035 (active-recall formats) done** on `feature/complete-m12`: STORY-0110 done ("Extract / Sync N card(s)…" on a note's page: preview of new/updated/unchanged items, sync into the note's stack with `extracted_from`/`extract_key` and a `Source: [[Note#Heading]]` link, idempotent, undoable; spec `docs/specs/inline-extraction.md`). STORY-0108 done (cloze gaps `{{c1::answer::hint}}` in cards and notes: masked in runs, groups take turns per review, highlighted answer, typed answers compared with the gap, toolbar button and Ctrl+Shift+C; spec `docs/specs/cloze.md`). STORY-0112 done (focus timer in the top bar on every screen, presets and custom lengths, blocks recorded against the overlapping calendar study session in `focus_block` (migration 26), "Start break" and "Log in Diary…"; spec `docs/specs/focus-timer.md`). STORY-0111 done (contrastive drill from a confused pair or 2-4 Library rows: match, compare side by side, keep the distinction as hints or a comparison note; spec `docs/specs/contrastive-drill.md`). STORY-0109 done (image occlusion cards: masks in an `occlusion` front-matter block, drawn on the card's page or started from a picture saved in the PDF viewer, Hide all / Hide one in runs; spec `docs/specs/image-occlusion.md`). Checked by tests and screenshots, not yet in an interactive window.
- **EPIC-034 (analytics) done** on `feature/complete-m12`: STORY-0107 done (true retention, retention by maturity and timing, forgetting curves per stability tier with the FSRS prediction and a fitted personal half-life; `report::retention_survival`, `ReportBundle.analytics`, Reports > Memory; 5.2 ms on 100k reviews). STORY-0102 and STORY-0103 done (Reports > Habits: hour/weekday profile with peak and low point, fatigue within a run; the slow-answer line in runs). STORY-0104 done (leeches and confusion pairs with open/pause/drill, Reports > Leeches and the stack Reports tab). STORY-0105 done (simulated forecast with stages, band and minutes; exam pacer; Reports > Workload and the stack pages). STORY-0106 done (workload against desired retention with the recommendation and one-click apply for the vault and each stack). STORY-0118 done (MCP leeches resource and report, study minutes, true retention curves; read-only checked again in tests and as the real binary), so EPIC-037 is done.
- **EPIC-033 (Exam mode) done** on `feature/complete-m12`: split into STORY-0125 to STORY-0129 (tasks TASK-0234 to TASK-0242), all done. `RunMode::Exam` (persisted `exam`, migration 25 with `exam_result`); the engine refuses flip, skip, rate, mark and blank answers and locks each answer (property tests); answers scored by the usual comparison, graded from what is stored (`studyfrog-service/src/exam.rs`), shown with percentage, grade, pass/fail and review; Reports > Runs, diary note, help page, control protocol (`rehearse type|submit|finish`). Checked by tests, screenshots of setup, run and result, the binary over pipes and `ctl`; not yet used by a person in a window. Exams never change the schedule (spec decision), so they do not feed familiarity.
- **STORY-0121 done (assistant as MCP client):** tool servers (program or HTTP address) on the Connections tab with discovery, consent for remote servers, and tool calls during wiki tasks and exams that wait for "Allow" unless trusted and read-only; tested against fake local servers (a shell script, an in-process router, a fake HTTP endpoint), never a real one. EPIC-037 in review only because STORY-0118 waits for EPIC-034.
- **STORY-0119 done (MCP authoring):** standard prompts as MCP prompts; staged authoring tools (items, inline cards, diary, calendar) reviewed and applied in the Assistant; destructive move/delete behind both flags with undo entries. Tested headless and with the egui harness, not with a real AI client.
- **STORY-0118 in review (MCP read-only part):** reports and diary resources, the schedule of an item per stack, `vault_get_due_queue`, `analytics_get_report`, `forecast_simulate_workload`, `pdf_read_pages`; tested on sample vaults, including that no file and no index row changes. Waits for EPIC-034: leech diagnostics, study-minute forecast, true retention curves.
- **STORY-0120 done:** `studyfrog --mcp --mcp-http 127.0.0.1:<port>` serves the MCP server on local HTTP (`POST /mcp`, 127.0.0.1 only, bearer token from `secrets.toml` or printed to standard error, Host and Origin checks); in-house HTTP reader in `studyfrog-net`; tested over a real local socket and as the real binary, not with a real AI client. No sessions or server-sent events.
- **STORY-0124 (control protocol) in review** on `feature/complete-m12`: `studyfrog --control-stdio [--vault PATH]` (headless, JSON Lines on standard input and output), `run/control.sock` served by the window and by headless instances (`run/control.json`, stale sockets replaced), `studyfrog ctl <command>`; `state`, `nav`, item/stack operations, rehearsal stepping, `query`, `search`, import/export, `screenshot` (window only), `logs --follow`; a policy like the MCP one (destructive needs `--control-allow-destructive`). Tested headless, over a real socket, as the real binary over pipes, and by driving a real window with `ctl`. Not built: exam runs, an AccessKit tree, the actions of calendar, board, diary, reports, settings and the assistant (reachable by `nav` only).
- **STORY-0096 (item keys) done** on `feature/complete-m12`: keys (`STACK_1`, `FLASHCARD_12`, ...) in front matter, given on creation; the Library's confirmed, undoable "Assign keys…" for older items; `key` in queries, exact keys in the top-bar and Library search; Key column, toolbar badge with "Copy key", Details row; migration 24. Not built: the `key:STACK_1..10` range form.
- **STORY-0097 and STORY-0123 (item colours) done:** `color` in front matter (palette word or quoted code), Color row in Details > Properties (swatches, None, picker, `#RRGGBB`); tinted item page, Details cards, rehearsal card and Library rows, contrast-checked (4.5:1) in every theme; checked by tests and screenshots (`--screen item:<ULID>`), not yet in an interactive window. EPIC-032 is in review (STORY-0098 still in review). Not built: Folders-view dot, bulk colour.
- **Work to finish before the next release (owner's instruction, 2026-11-08): complete every open M12 issue first.** Branch `feature/complete-m12` (from `main` at v0.5.0; push commits there, **tag nothing** until all are done). 0.5.0 was published with part of the scope; 31 issues are still in `backlog` (see BOARD.md) and 10 are in `review`. Order in `priority-list.md` ("Phase M12"). How to continue, from what worked and what did not:
  - **Gate:** `scripts/check.sh` takes about 8 minutes. Run it detached (`setsid nohup ...`) with `STUDYFROG_HOME` set to a scratch directory so tests never touch the real home, and poll the log; a foreground command is cut at 10 minutes.
  - **Limits that bit:** files are at most 250 lines (rustfmt grows test files; put shared test helpers in `tests/<name>_common/mod.rs`); `effect.rs` and `exec.rs` are at the limit, so a new effect goes into an existing sub-enum (for example `TransferJob`); clippy forbids `thread::sleep` and long function-pointer types (use aliases); `/home` is btrfs and its metadata filled up once (`cargo clean -p` of the workspace crates fixed it).
  - **Item keys (STORY-0096) and colour (STORY-0097, STORY-0123):** read and write them as unknown front-matter keys (`studyfrog_md::extra`, preserved verbatim) rather than touching the typed schema of all four kinds; add an index column in a new store migration; allocate keys on item creation and offer an explicit "assign keys to existing items" action instead of rewriting a vault silently. The `priority` and `difficulty` fields touch about 50 files and are the model for query support (`core/src/query_fields.rs`).
  - **Exam mode (EPIC-033)** is done (STORY-0125 to STORY-0129); the UI part lives in `exam_ui.rs`, `view/exam_card.rs`, `view/exam_result.rs`, `control_exam.rs`.
  - **MCP:** the engine is `crates/studyfrog-agent/src/mcp_*.rs`; tools and resources are registered from `crates/studyfrog-ui/src/mcp_vault.rs` (one module per group: `mcp_vault_read`, `mcp_search`, `mcp_reports`, `mcp_diary_pdf`, `mcp_write`, `mcp_structural`, `mcp_prompts`). To finish STORY-0118 after EPIC-034: answer `analytics_get_report` with `kind: leeches`, fill `study_minutes` in `forecast_simulate_workload`, and use the true retention curve in `mcp_reports.rs`. Keep `State`-handling update functions small and edit the owned state in place: debug-build frames of `step` and `agent_ui::update` are about 600 KB each, and the test threads have 2 MB.
  - **Control protocol (STORY-0124)** is in review: commands are parsed in `control_words.rs`, turned into phases of messages in `control_plan.rs`; add a command there and in `control_help.rs`.
- **0.5.0 released (2026-11-08)** with part of phase M12: the application home and logs, the MCP server (read-only), Gemini and DeepSeek, the prompt library and screen, 32 achievement tiers with a per-stack view, the reports as CSV, and the PDF fixes. About a third of the planned stories; the rest moves on (see `priority-list.md` and `docs/RELEASE_NOTES_0.5.0.md`). Everything is gate-checked; the new screens were tested headless only. **Next:** the unbuilt M12 stories in priority order (control protocol `STORY-0124`, item keys and colours, Exam mode, analytics, active-recall formats, UI refinements, Python API), and a real-window check of this release.
- **EPIC-037 (MCP) done:** STORY-0117 to STORY-0121 done; STORY-0118 finished on 2026-11-08 with the EPIC-034 reports. Run as the real binary over pipes; not tried with a real AI client or a real external MCP server.
- **STORY-0098 (reports as CSV) in review:** "Export as CSV" on the Reports screen writes `exports/studyfrog-report-<vault>-<date>.csv`; no per-stack statistics or learned and mastered totals yet; no save dialog; tested headless.
- **EPIC-029 (achievements) built:** 32 built-in tiers (STORY-0086, done) and the filter, grouping and per-stack summary (STORY-0087, in review, tested headless only).
- **STORY-0083 (Google Gemini) and STORY-0084 (DeepSeek) in review:** two more providers of the assistant with their wire formats, default addresses and suggested models; tested against fixed requests and answers only, never against the real endpoints; no explicit "test connection" button yet.
- **EPIC-030 (prompt expansion) built:** prompts `explain`, `exam`, `socratic` and `concept_map` (STORY-0088 to 0091, done); categories, search and options in the prompts (STORY-0092) and the Prompts screen (TASK-0233) are in review, tested headless only.
- **STORY-0122 (application files in `~/.local/share/studyfrog`, logs) in review:** config, models and logs live in one home (`STUDYFROG_HOME` overrides); the old `~/.config/studyfrog` is copied in, never moved; every notice goes to `logs/studyfrog.log` and `logs/console.jsonl` (5 MB x 3); `studyfrog --print-paths` and `studyfrog logs [--tail N] [--follow] [--json]`; Settings > Data lists the paths. Not done: non-notice events, "Open logs folder", `run/` socket. Not tried on a real `~/.config`. Details in the story.
- **STORY-0085 (PDF viewer performance) in progress:** page and thumbnail pictures are trimmed apart (6 and 60) and share one thumbnail scale; benchmarks show parsing is about 10 % of a render, so a per-session parsed document is not built. Real-window 60 fps check and true LRU are open. Details in the story.
- **BUG-0010 (PDF pages illegible at some zoom or fit modes) in review:** page textures are keyed by pixel size, so a thumbnail shown while the sharp render is in flight no longer takes over the sharp texture; unit tests added, `scripts/check.sh` passes. Not yet seen in a real window. Next in `priority-list.md`: STORY-0085 (PDF viewer performance).
- **TASK-0232 (Documentation, help and release notes for the PDF export) done:** in-app help topic `pdf-export` and FAQ entries explaining export routes, options, and supported/unsupported elements; complete showcase example in `examples/pdf-export/complete-note.md`; end-to-end service test verifying text extraction of all elements; manual verification in a real window; draft release notes for 0.4.0 in `docs/RELEASE_NOTES_0.4.0.md`.
- **TASK-0231 (Export to PDF in the application) done:** PDF export modal dialog with page size, margins, font size, title page, TOC, page numbers, and card/question answers; live preview before export; accessible entry points on item page, library bulk bar, stack page, and Transfer dialog; notifications and toasts opening exported PDFs in the viewer.
- **TASK-0174 (Self-assessment against measured results) done:** pure calibration report in `report` comparing subjective diary ratings (understanding, retention, focus, mood) against measured review accuracy and familiarity; over/underconfidence bias analysis; "Self-assessment" tab in Reports screen with metrics, sample size indicators, and timeline.
- **TASK-0173 (Prompt for an entry after a rehearsal) done:** compact post-rehearsal self-assessment prompt in the run summary pre-filling duration, linked run ULID, stack, and reviewed item counts; exposes 1..=5 primary scale buttons with level tooltips; quick reflection field; single-key dismissal (`Esc`); non-blocking exit.
- **TASK-0230 (Export use cases for PDF in the service) done:** pure export planning and safe execution in `studyfrog-service` for single items, selected item sets, and stacks (including sub-stacks and deduplicated referenced items); resolves internal wikilinks and picture attachments with missing-picture placeholders; atomic writing via temp file; hostile path protection; cooperative cancellation and progress reporting.
- **TASK-0172 (Diary screen) done:** full Diary screen with list and month-calendar views, rich entry draft editing form, 13 standard 1–5 self-assessment scales with level descriptions, user-defined custom scales in Settings, stack/rating/tag filters, date navigation, and delete confirmations.
- **TASK-0229 (Markdown layout and PDF writer) done:** pure layout engine and lopdf PDF writer in `crates/studyfrog-pdf/src/write/` paginating Markdown AST into pages with wrapped text, code block syntax highlighting, aligned tables, lists, blockquotes, bookmarks, web/internal wikilinks, and accessible ToUnicode CMaps.
- **TASK-0204 (Folders view and Items view in the application) done:** Folders screen modelled on Obsidian's file explorer with toolbar (+Note, +Folder, sort order, expand/collapse), virtualized indent tree, kind badges, context menus, drag & drop, cross-stack move confirmation, F2 inline rename, keyboard shortcuts, and full accessibility.
- **TASK-0171 (Diary entries in the data model, Markdown and index) done:** `DiaryEntry` types in `core`; `kind: diary` frontmatter parsing, rendering, and lossless roundtrip in `md`; SQLite migration 23 and index tables with day-listing, text search, and item linkage in `store`.
- **TASK-0203 (Folder operations in the service) done:** pure plans and atomic execution with undo snapshots for create, rename, move, and delete inside the vault; link and image attachment rewriting; cross-stack boundary primary stack transitions and `referenced_by` updates; hostile path protection; tests pass.
- **TASK-0170 (Specify the diary) done:** authored `docs/specs/diary.md` with entry format, 13 standard 1–5 self-assessment scales, custom scales in TOML, bidirectional links, and post-rehearsal prompts.
- **TASK-0202 (Specify the folders and items views) done:** authored `docs/specs/vault-views.md` defining the Obsidian-modelled Folders view and structured Items view.
- **TASK-0228 (Specify the Markdown to PDF export and choose the writer) done:** authored `docs/specs/pdf-export.md` and `docs/adr/0012-markdown-to-pdf-writer.md` selecting in-house layout on `lopdf`.
- **TASK-0220 (Export formats TOML and RON next to JSON) done:** export dialog offers JSON, TOML and RON with advanced per-stack scheduling values.
- **Next up:** TASK-0195 (Calm the remaining screens).
- **Verified by hand in a window:** PDF export dialog options, live preview, and generated 4-page showcase PDF were verified in a real window environment on Linux (with X11/Wayland).
- **Released:** 0.2.0 (notes in `docs/RELEASE_NOTES_0.2.0.md`). EPIC-022 (PDF
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
  unverified from memory. The remote `origin` exists (github.com/louisgoodnews/study-frog); CI has not run on this code yet.

## Next
- **0.3.0 and 0.3.1 (phase M10) are released.** **Now (0.4.0, phase M11), built and committed on local `main`, not pushed and not released (the owner wants the whole priority list done first):** appearance settings, navigation history, list continuation, PDF names, auto-save with the "Saved" toast, Open on the created notice, import formats and examples, CSV mapping, standard prompts and the Prompts tab, running a prompt through the assistant, tables, underline, colour, a table of contents, code highlighting, text macros, ADR 0011, `referenced_by` with the `membership` view (schema v20), and the services, library column, JSON, CSV and import following the membership (TASK-0212).
- **Done:** TASK-0218 (memory per stack: schema v21 and v22, ratings, queues, daily counts, Home, calendar, rerate) and TASK-0219 (familiarity and difficulty per stack, averaged on the item). Screens that show one value per item (status, due column, reports' item lists) still read the primary stack's schedule through the views `mem_primary`, `sm2_primary` and `review_once`.
- **Next, in `priority-list.md` order:** TASK-0213 (references in the interface, in progress), TASK-0217 (mark a folder as a stack), TASK-0220 (TOML and RON export), then the views (TASK-0202 spec, TASK-0203, TASK-0204), TASK-0195 (calm the remaining screens), the diary (TASK-0170 to TASK-0174), then the release steps of `docs/contributing/RELEASING.md` (version bump to 0.4.0, notes with the install steps, AppImage, tag, push).
- **Open points for the owner:** ADR 0011 is marked accepted on the strength of the owner's decisions of 2026-11-03; the wording is Claude's. Not built or not tried: attaching PDF pages to a prompt run, a mapping remembered per file name, `also_in` in the query language. The auto-save, prompts, tables and macros were tested headless only.
- **Not verified by hand:** the owner tried the title bar and the new layout on their GNOME/Wayland machine and was satisfied; the progress bar was seen at 0 done only (partial fill and repeats are tested headless). The benchmarks were not re-run for 0.3.1.
- **Open flag:** the help outline (`docs/help/README.md`, TASK-0161) shipped at the owner's release instruction without a separate review of the outline; the help screen and tutorial card were tested headless only.
- **0.3.0 is planned** (phase M10, EPIC-023, `docs/roadmap/M10-usability-and-insight.md`): 15 stories and one bug (BUG-0001, commas in teacher names), all in backlog.
  The owner answered the open design questions on 2026-11-02 (accepted choices are listed in the phase file); what is still
  open is in each story's Open Questions. The familiarity and FSRS model is researched and decided (`docs/research/familiarity-and-fsrs.md`: relative factor, on from the start).
  Priority order in four waves is in the phase file; wave 1 is `ready` (BUG-0001 is done; the list is `priority-list.md`): TASK-0143 (statuses spec), TASK-0142 (settings
  tabs), TASK-0128 (vault name), TASK-0135 (familiarity spec).
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
