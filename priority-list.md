# Priority list

The order in which Claude works on the planned items of phase M10 (0.3.0), kept by hand next to the generated
[BOARD.md](BOARD.md). The reasons and the dependencies are in
[docs/roadmap/M10-usability-and-insight.md](docs/roadmap/M10-usability-and-insight.md). Tick an item when it is done and
verified (`scripts/check.sh` passes), and report it to the owner. The owner may reorder the list at any time.

Last updated: 2026-11-24 (613 of 626 issues done; 13 open, all in EPIC-067; v1.0.0 awaiting packaging/owner benchmarks/tag).

## Wave 1: start now
- [x] 1. [BUG-0001](docs/issues/bugs/BUG-0001-teacher-and-subject-names-with-a-comma-cannot-be-used.md) Names with commas cannot be used
- [x] 2. [TASK-0143](docs/issues/tasks/TASK-0143-specify-the-statuses-their-names-and-their-transitions.md) Specify the statuses
- [x] 3. [TASK-0142](docs/issues/tasks/TASK-0142-settings-tab-bar-and-domain-grouping.md) Settings tab bar
- [x] 4. [TASK-0128](docs/issues/tasks/TASK-0128-vault-name-next-to-the-studyfrog-name-in-the-bar.md) Vault name in the bar
- [x] 5. [TASK-0135](docs/issues/tasks/TASK-0135-specify-the-familiarity-value.md) Specify the familiarity value

## Wave 2: foundations
- [x] 6. TASK-0144 and TASK-0145 Statuses in the data model and the UI
- [x] 7. TASK-0136 Record view and action time
- [x] 8. TASK-0137 Familiarity as a pure function, and TASK-0141 its factor on the FSRS interval (on from the start)
- [x] 9. TASK-0126 and TASK-0127 Notification records and the notification center
- [x] 10. TASK-0129 Specify the library redesign and the query syntax
- [x] 11. TASK-0179 and TASK-0180 Rehearsal modes: specification and engine

## Wave 3: the big screens
- [x] 12. TASK-0130 to TASK-0134 Library: query, list, search bar, saved filters, forms out of the page
- [x] 13. TASK-0146 to TASK-0148 Calendar: plan the next reviews, move them, drag and drop
- [x] 14. TASK-0138 to TASK-0140 and TASK-0184 Familiarity in reports, Home and queues, and the replay
- [x] 15. TASK-0149 to TASK-0151 Stack overview: specification, shell, Summary

## Wave 4: the rest
- [x] 16. TASK-0152 to TASK-0156 Stack sub-views
- [x] 17. TASK-0181 and TASK-0182 Rehearsal mode screens and reports
- [x] 18. TASK-0157 to TASK-0160 Import and export, then TASK-0167 to TASK-0169 Anki and Obsidian
- [x] 19. TASK-0161 to TASK-0166 Help: wiki, FAQ, tutorials
- [ ] 20. ~~TASK-0170 to TASK-0174 Diary~~ moved to 0.4.0 (see below)
- [x] 21. TASK-0185 to TASK-0187 Create dialog like Jira and defaults for new items

## Next: phase M11 (0.4.0)
Proposed by Claude on 2026-11-03 after the owner's test of 0.3.1; the owner may reorder. Reasons and dependencies are in
[M11-diary](docs/roadmap/M11-diary.md).

**Wave 1: small and painful now**
- [x] BUG-0009 PDF lists name the file (done)
- [x] 1. BUG-0007 Enter in a list does not continue it, BUG-0008 Bullets sit higher than their text
- [x] 2. TASK-0199 Open the new item from its "created" notification
- [x] 3. TASK-0200 Auto-save token by token (on a whitespace, at the latest after 5 seconds) and no Save button, then TASK-0201 "Saved" toast when leaving the editor
- [x] 4. TASK-0209 Tell the user which layouts and formats the importers read

**Wave 2: new capabilities**
- [x] 5. TASK-0208 CSV import with column mapping (all problems at once)
- [x] 6. TASK-0205 Write the standard prompts (they hand-hold an AI to files StudyFrog reads), TASK-0206 Prompts screen (copy and export)
- [x] 7. TASK-0197 Tables, TASK-0215 Underline and colour, TASK-0216 Table of contents, TASK-0198 Text macros, TASK-0214 Code highlighting
- [x] 8. TASK-0207 Run a prompt with the assistant
- [x] 8b. EPIC-027 Appearance: TASK-0221 (spec), TASK-0222 (dark, light or system), TASK-0223 to TASK-0227 are all built; the check of every screen in both themes (TASK-0226) is done except the dialogs and the PDF viewer

**Wave 3: larger, specify first**
- [x] 9. TASK-0210 Decide the stack folder and reference rules (ADR; the owner: a folder can be marked as a stack, an item can be referenced by several stacks)
- [x] 10. TASK-0211 to TASK-0213, TASK-0217 to TASK-0220, TASK-0202 to TASK-0204 (all done: references, stack marks, per-stack schedules, TOML/RON exports, Folders and Items views)
- [x] 11. TASK-0195 Calm the remaining screens (STORY-0073)
- [x] 11b. TASK-0228 to TASK-0232 Export notes to PDF (all done: spec/ADR, layout/writer, service, UI, docs/help/notes)
- [x] 12. TASK-0170 to TASK-0174 Diary (all done: spec, model/index, screen, rehearsal prompt, calibration report)
- [x] TASK-0196 Navigation history, BUG-0005 Exit button cut off, BUG-0006 Roadmap too small (done on main after 0.3.1)

## Feedback after 0.3.0 (released as 0.3.1 on 2026-11-03, EPIC-025)
- [x] BUG-0002 Padding, BUG-0003 Create dialog header, BUG-0004 Home figures
- [x] TASK-0188 Ask before discarding a filled Create dialog
- [x] TASK-0189 Red Exit button
- [x] TASK-0191 Suggest Anki and Obsidian files to import
- [x] TASK-0190 Own title bar (the owner tried it on 2026-11-03)
- [x] TASK-0192 Calmer UI, first part: library, stack page, Home, rehearsal setup, settings and help (`docs/specs/ui-calm.md`); accepted by the owner, the rest is TASK-0195
- [x] TASK-0193 Space between content and scroll bars, TASK-0194 Rehearsal progress bar against the open items

## Shipped in 0.4.1 – 0.4.3 (EPIC-031 & follow-ups)
- [x] [STORY-0093](docs/issues/stories/STORY-0093-details-view-notebook-overhaul.md) Details view notebook overhaul with tabs and stack due date editing
- [x] [STORY-0094](docs/issues/stories/STORY-0094-create-menu-due-date-and-priority.md) Create menu due date and priority assignment for stacks and other creatable items
- [x] [STORY-0095](docs/issues/stories/STORY-0095-note-creation-modal-expansion-and-write-view.md) Note creation modal full-window expansion and default write view
- [x] [STORY-0099](docs/issues/stories/STORY-0099-simplify-vault-switcher-menu.md) Simplify vault switcher menu
- [x] [STORY-0100](docs/issues/stories/STORY-0100-move-stack-between-parent-and-standalone.md) Move stack between parent stack and standalone after creation
- [x] [STORY-0101](docs/issues/stories/STORY-0101-delete-item-from-viewer-and-details.md) Delete item from item viewer and editor
- [x] [BUG-0011](docs/issues/bugs/BUG-0011-importer-only-accepts-tsv-and-fails-on-csv-prompt-outputs.md) Importer auto-detects CSV/TSV format
- [x] [BUG-0012](docs/issues/bugs/BUG-0012-rehearsal-card-text-wrapping-and-unquoted-csv-comma-truncation.md) Rehearsal card text wrapping and unquoted CSV comma recovery
- [x] [BUG-0013](docs/issues/bugs/BUG-0013-rehearsal-result-view-scrollability-review-stretch-and-suggested-rating.md) Rehearsal result view scrollability, stretched review area, and suggested rating visibility
- [x] [BUG-0014](docs/issues/bugs/BUG-0014-csv-import-fails-on-unquoted-multiline-fields.md) CSV importer recovery on unquoted multiline fields
- [x] [BUG-0015](docs/issues/bugs/BUG-0015-csv-escaped-newlines-and-card-title-truncation.md) Escaped newline sequences in imported cards and 60-character card title truncation

## Phase M12 (released in 0.5.0 and 0.6.0)
Details in [docs/roadmap/M12-models-and-extensibility.md](docs/roadmap/M12-models-and-extensibility.md) and [PLANNED_FEATURES.md](PLANNED_FEATURES.md).

**Wave 1: PDF legibility & performance, application files (`~/.local/`), and CLI control protocol**
- [x] 1. [BUG-0010](docs/issues/bugs/BUG-0010-pdf-pages-are-illegible-depending-on-zoom-or-page-fit-mode.md) PDF pages are illegible depending on zoom or page fit mode, and [STORY-0085](docs/issues/stories/STORY-0085-performance-optimization-in-studyfrog-pdf-viewer.md) PDF viewer 60 fps performance optimization — **0.5.0:** the texture fix (BUG-0010) and separate page and thumbnail caches are in; open: a check in a real window, LRU by use. A parsed-document cache was measured and not built (parsing is ~10 % of a render). **Done 2026-11-08:** true LRU by last use, frame-time and thumbnail-request tests with a 500-page file (median 8.3 ms per frame in a debug build), automated sharpness check and real-binary screenshots at every fit mode and several zooms.
- [x] 2. [STORY-0122](docs/issues/stories/STORY-0122-studyfrog-application-files-in-local-with-logs-and-configurations.md) StudyFrog application files in `~/.local/` with rotating logs and configurations — **0.5.0:** built (STORY-0122): app home, rotating logs, `--print-paths`, `logs`, Settings paths. Open: other worker events in the log, an "Open logs folder" button. (`run/control.sock` exists since STORY-0124; control-protocol events are logged.) **Done 2026-11-08:** both built and verified.
- [x] 3. [EPIC-038](docs/issues/epics/EPIC-038-cli-control-protocol-for-interactive-ai-agent-application-operation.md) / [STORY-0124](docs/issues/stories/STORY-0124-interactive-cli-control-protocol-and-console-log-streaming-for-ai-agents.md) Interactive CLI control protocol and console log streaming for AI agents — **In review (2026-11-08):** `--control-stdio`, `run/control.sock` (window and headless), `studyfrog ctl`, `state`, navigation, item and stack operations, rehearsal stepping, query, import/export, screenshots (window) and `logs --follow`; exam runs since STORY-0129; not yet every screen's actions or an AccessKit tree (see the story). **2026-11-23:** the board, diary, the nine report tabs and the calendar views are driven and answer with their data; **2026-11-23 (later):** achievements, graph, assistant, settings, match, folders, PDF open and stack delete are driven, and the window's widget tree answers `ui tree`, `ui find` and `ui click`. EPIC-038 and STORY-0124 are done (verified in the running app, see STORY-0124). The log part is in STORY-0122.

**Wave 2: Item keys, colorization, Exam run mode, and UI psychophysics**
- [x] 4. [EPIC-032](docs/issues/epics/EPIC-032-item-keys-colorization-and-reporting-export.md): [STORY-0096](docs/issues/stories/STORY-0096-item-keys-for-querying.md) Item keys (`STACK_1`, `FLASHCARD_500`), [STORY-0097](docs/issues/stories/STORY-0097-item-colorization-with-color-picker-and-viewer-background-styling.md) & [STORY-0123](docs/issues/stories/STORY-0123-colorization-field-for-items-influencing-details-and-rehearsal-background.md) Item colorization field influencing Details and Rehearsal view backgrounds, and [STORY-0098](docs/issues/stories/STORY-0098-reporting-export-as-csv.md) Reporting export as CSV — **Epic done (2026-11-08: STORY-0098 has per-stack statistics, totals, streak and a folder choice).** **0.5.0:** STORY-0098 (CSV export, no per-stack statistics). **feature/complete-m12:** STORY-0097 and STORY-0123 done (colour in front matter, picker in Details, tinted item page, Details, rehearsal card and Library rows); STORY-0096 done (keys on creation, confirmed "Assign keys…", `key` in queries and searches, shown in the list, toolbar and Details).
- [x] 5. [EPIC-033](docs/issues/epics/EPIC-033-exam-run-mode-with-semantic-comparison-and-grading.md) Exam run mode with mandatory text entry, semantic comparison, and percentage grading (`60%` pass threshold `D`/`4.0`, `5%` bands up to `95%–100%` `A+`/`1+`) — **Done (2026-11-08) on `feature/complete-m12`:** STORY-0125 to STORY-0129 (spec, engine rules, scoring and results, screens, help, control protocol); verified by tests and screenshots, not yet by a person in a window.
- [x] 6. [EPIC-036](docs/issues/epics/EPIC-036-evidence-based-ui-aesthetics-perceptual-color-science-and-ux-refinement.md): [STORY-0113](docs/issues/stories/STORY-0113-perceptual-oklch-color-palette-calibration-and-apca-contrast-verification.md) OKLCH/APCA color calibration, [STORY-0114](docs/issues/stories/STORY-0114-psychophysical-typographic-measure-modular-hierarchy-and-tabular-numerals.md) 60–75 cpl typographic measure & tabular numerals, [STORY-0115](docs/issues/stories/STORY-0115-low-entropy-surface-elevation-border-decluttering-and-gestalt-spacing.md) border de-cluttering & Gestalt spacing, and [STORY-0116](docs/issues/stories/STORY-0116-distraction-free-zen-focus-mode-and-zero-layout-shift-interaction-polish.md) Zen focus mode — **Built 2026-11-08; STORY-0113, 0115, 0116 done, STORY-0114 in review (reading measure in characters, not the pixel width named), so the epic was in review. **2026-11-23:** done: Split sits under the text ("Preview below"), so every view keeps the 55 to 75 measure; the epic and STORY-0114 are done. Earlier: verified in the app: criteria 1 (APCA palettes) and 3 (tonal surfaces, Zen mode) hold; criterion 2 does not hold for note editing (the Markdown editor is full width), so it stays open.**

**Wave 3: Behavioral learning analytics, FSRS workload forecasting, and active-recall formats**
- [x] 7. [EPIC-034](docs/issues/epics/EPIC-034-behavioral-learning-analytics-fsrs-optimizer-and-workload-forecasting.md): [STORY-0102](docs/issues/stories/STORY-0102-chronobiological-hour-of-day-and-day-of-week-performance-telemetry.md) Chronobiological 24h/7d telemetry, [STORY-0103](docs/issues/stories/STORY-0103-within-session-cognitive-fatigue-and-pacing-decay-analysis.md) Within-session fatigue decay, [STORY-0104](docs/issues/stories/STORY-0104-leech-diagnostics-interference-detection-and-confusion-clusters.md) Leech & interference diagnostics, [STORY-0105](docs/issues/stories/STORY-0105-fsrs-state-simulation-forecasting-study-minute-projection-and-exam-pacer.md) FSRS state-simulation forecast & Exam Deadline Pacer, [STORY-0106](docs/issues/stories/STORY-0106-workload-versus-retention-simulator-and-minimum-recommended-retention.md) CMRR Workload vs. Retention simulator, and [STORY-0107](docs/issues/stories/STORY-0107-empirical-forgetting-curves-true-retention-and-stability-tier-survival-analysis.md) True Retention & stability-tier forgetting curves — **Not built, moves on.** **2026-11-08: done** on `feature/complete-m12`: all six stories built and verified (Reports > Memory, Habits, Leeches, Workload; stack pages; the slow-answer line in runs).
- [x] 8. [EPIC-035](docs/issues/epics/EPIC-035-rich-active-recall-formats-inline-note-extraction-and-focus-workflows.md): [STORY-0108](docs/issues/stories/STORY-0108-inline-cloze-deletions-in-flashcards-and-notes.md) Inline Cloze deletions, [STORY-0109](docs/issues/stories/STORY-0109-image-occlusion-cards-for-diagrams-slides-and-pdf-figures.md) Image Occlusion cards, [STORY-0110](docs/issues/stories/STORY-0110-inline-flashcard-and-question-extraction-from-markdown-notes.md) Inline Note-to-Card extraction, [STORY-0111](docs/issues/stories/STORY-0111-contrastive-side-by-side-drill-for-confused-and-interfering-items.md) Contrastive side-by-side drill, and [STORY-0112](docs/issues/stories/STORY-0112-pomodoro-focus-timer-with-automatic-learning-diary-and-session-logging.md) Pomodoro focus timer — **Not built, moves on.** **2026-11-08:** STORY-0110 done (inline cards synced from a note). STORY-0108 done (cloze deletions). STORY-0112 done (focus timer). STORY-0111 done (contrastive drill). STORY-0109 done (image occlusion); EPIC-035 done.

**Wave 4: MCP server/client, external AI providers, prompt expansion, Python API, and achievements**
- [x] 9. [EPIC-037](docs/issues/epics/EPIC-037-state-of-the-art-model-context-protocol-mcp-server-client-and-registry.md): [STORY-0117](docs/issues/stories/STORY-0117-pure-json-rpc-2-0-mcp-protocol-engine-and-plain-data-registry.md) to [STORY-0121](docs/issues/stories/STORY-0121-built-in-assistant-mcp-client-host-for-external-academic-tool-servers.md) State-of-the-art MCP Server (`studyfrog-app --mcp`), plain-data `McpRegistry`, and Assistant MCP Client Host — **0.5.0, partly:** STORY-0117 done; STORY-0118 (read-only resources and search) and STORY-0120 (stdio and config block) partly. **Moves on:** reports, diary and more tools, STORY-0119, STORY-0121. **2026-11-08:** STORY-0120 done (the localhost HTTP transport with bearer token and Host/Origin checks). STORY-0118 in review: reports, diary, item schedules and the due-queue, analytics, forecast and PDF tools built; leeches, study minutes and true retention curves wait for EPIC-034. STORY-0119 done (prompts, staged authoring tools, destructive tools with undo). STORY-0121 done (the assistant uses external MCP tool servers with approval). EPIC-037 in review until STORY-0118 is done. **2026-11-08:** STORY-0118 done (leeches, study minutes, true retention over MCP); EPIC-037 done.
- [x] 10. [EPIC-028](docs/issues/epics/EPIC-028-extensibility-plugin-modding-and-python-api.md): [STORY-0083](docs/issues/stories/STORY-0083-google-gemini-in-models-for-studyfrog.md) Google Gemini, [STORY-0084](docs/issues/stories/STORY-0084-deepseek-in-models-for-studyfrog.md) DeepSeek, and Python modding API — **0.5.0, partly:** STORY-0083 and STORY-0084 (no real-endpoint run). **Moves on:** the Python modding API. **2026-11-08:** the Python modding API is built on `feature/complete-m12` (ADR 0013; STORY-0130 lifecycle events, STORY-0131 Python client, STORY-0132 script runner with cron and event-bound jobs, STORY-0133 Console screen, STORY-0134 docs and examples, all done). The item stays open for STORY-0085 (PDF performance, another package) and the review of STORY-0083/0084. **Done 2026-11-08:** STORY-0085 done; STORY-0083 and STORY-0084 done with "Test connection" (offline checks only).
- [x] 11. [EPIC-030](docs/issues/epics/EPIC-030-prompt-expansion.md): [STORY-0088](docs/issues/stories/STORY-0088-multi-level-file-explanation-prompt.md) to [STORY-0092](docs/issues/stories/STORY-0092-prompt-library-categorization-and-parameterized-execution-ui.md) and [TASK-0233](docs/issues/tasks/TASK-0233-move-prompt-library-to-standalone-left-sidebar-navigation.md) Prompt library expansion and standalone sidebar navigation
- [x] 12. [EPIC-029](docs/issues/epics/EPIC-029-expanded-achievements-and-per-stack-progression.md): [STORY-0086](docs/issues/stories/STORY-0086-expanded-achievement-catalog-and-milestone-tiers.md) and [STORY-0087](docs/issues/stories/STORY-0087-per-stack-achievement-ui-and-filtering.md) Expanded achievements and per-stack progression

## Next: phase M13 (0.7.0)

**Wave 1: Item linking & graph visuals**
- [x] 1. [EPIC-039](docs/issues/epics/EPIC-039-universal-item-linking-between-stacks-cards-questions-answers-and-notes.md) Universal item linking between stacks, cards, questions, answers, and notes (bidirectional indexing, kind-badged chips, hover previews, autocomplete picker, and Relations tab overhaul)
  - [x] [STORY-0135](docs/issues/stories/STORY-0135-comments-for-items.md) Comments for items (YAML front-matter persistence, SQLite comment indexing, search queries, notebook Comments tab, and CLI/MCP controls)

**Wave 2: Flexible rehearsal presentation & active recall**
- [x] 2. [EPIC-040](docs/issues/epics/EPIC-040-intermittent-typed-active-recall-and-flexible-rehearsal-presentation.md) Intermittent typed active recall and flexible rehearsal presentation
  - [x] [STORY-0136](docs/issues/stories/STORY-0136-make-typed-answers-random-per-selection-during-rehearsal.md) Make typed answers random per selection during rehearsal (rehearsal setup sampling setting, notes excluded from typed answers, dynamic card view alternating between typed submission and flip)

**Wave 3: Modularity & system-driven architecture**
- [x] 3. [EPIC-041](docs/issues/epics/EPIC-041-modularity-system-driven-architecture-and-decoupled-background-services.md) Modularity: system-driven application architecture and decoupled background services (file surveys on vault open/switch rather than lazy view open, decoupled domain services)

**Wave 4: Interactive AI content generation & prompt library**
- [x] 4. [EPIC-042](docs/issues/epics/EPIC-042-interactive-ai-content-generation-and-parameter-guided-prompt-workflows.md) Interactive AI content generation and parameter-guided prompt workflows
  - [x] [STORY-0137](docs/issues/stories/STORY-0137-interactive-parameter-guided-content-generation-prompts.md) Interactive parameter-guided content generation prompts in prompt library (intake interview for relevance, language, difficulty, and style choices)

**Wave 5: Navigation, profile & file management UX overhaul**
- [x] 5. [EPIC-043](docs/issues/epics/EPIC-043-navigation-profile-and-file-management-ux-overhaul.md) Navigation, profile, and file management UX overhaul
  - [x] [STORY-0138](docs/issues/stories/STORY-0138-overhaul-profile-and-vault-switchers.md) Overhaul of profile and vault switchers (elevated card layouts, active indicators, disambiguated paths, profile avatars, safe creation/deletion dialogs)
  - [x] [STORY-0139](docs/issues/stories/STORY-0139-overhaul-file-navigator-and-dialogue.md) Overhaul of file navigator and in-app file dialogue (indent guides, file type icons, inline search, path breadcrumbs, bookmark shortcuts)
  - [x] [STORY-0140](docs/issues/stories/STORY-0140-rehearsal-and-exam-comments-drawer.md) Expandable comments drawer in rehearsal and exam modes (slide-out drawer or panel to view and append item comments during active rehearsal/exam runs)

**Bug fixes & calibration refinements**
- [x] [BUG-0016](docs/issues/bugs/BUG-0016-metacognitive-calibration-accuracy-baseline-and-default-item-difficulty-priority.md) Metacognitive calibration measured accuracy baseline calculation and default item difficulty (Hard) and priority (Medium)
- [x] [BUG-0017](docs/issues/bugs/BUG-0017-markdown-viewer-bold-text-visibility.md) Boldness of words in markdown viewer visible with distinct contrast
- [x] [BUG-0018](docs/issues/bugs/BUG-0018-code-block-indentation-preservation.md) Code block indentation preservation with tab stop expansion
- [x] [BUG-0019](docs/issues/bugs/BUG-0019-notes-stack-membership-optional.md) Notes in stacks optional (loose notes at vault root permitted)

**Wave 6: Markdown viewing, macro library expansion, and brand mascot icon**
- [x] [EPIC-045](docs/issues/epics/EPIC-045-macro-library-expansion-and-escaping-syntax.md) Macro library expansion & escaping syntax
  - [x] [STORY-0143](docs/issues/stories/STORY-0143-comprehensive-markdown-macro-library-expansion.md) Comprehensive markdown macro library expansion
  - [x] [STORY-0144](docs/issues/stories/STORY-0144-macro-escaping-syntax-to-disable-expansion.md) Macro escaping syntax ({#...}) to disable expansion
- [x] [EPIC-046](docs/issues/epics/EPIC-046-markdown-viewer-list-syntax-enhancements.md) Markdown viewer list syntax enhancements
  - [x] [STORY-0145](docs/issues/stories/STORY-0145-markdown-list-parser-support-for-dash-asterisk-numbers-and-alphabetical-markers.md) Markdown list parser support for dash, asterisk, numbers and alphabetical markers
- [x] [STORY-0146](docs/issues/stories/STORY-0146-stylized-breviceps-macrops-mascot-icon.md) Stylized Breviceps macrops brand mascot icon

**Wave 7: Timetable planner, calendar daily/weekly views, floating burger navigation & scriptrunner console**
- [x] [EPIC-044](docs/issues/epics/EPIC-044-timetable-schedule-planner-and-daily-weekly-calendar-views.md) Timetable schedule planner & daily/weekly calendar views
  - [x] [STORY-0141](docs/issues/stories/STORY-0141-timetable-schedule-planner-for-lectures-and-lessons.md) Timetable schedule planner for lectures and lessons
  - [x] [STORY-0142](docs/issues/stories/STORY-0142-calendar-daily-and-weekly-views.md) Calendar daily and weekly views
- [x] [STORY-0147](docs/issues/stories/STORY-0147-research-floating-burger-navigation-menu.md) Research floating burger navigation menu
- [x] [EPIC-048](docs/issues/epics/EPIC-048-scriptrunner-environment-and-internal-automation-api.md) / [STORY-0148](docs/issues/stories/STORY-0148-console-expansion-to-full-scriptrunner-environment-with-internal-api.md) Console expansion to full scriptrunner environment with internal automation API

## Next: phase M14 (0.8.0)

**Wave 1: Scripting studio rebranding, DSL research, job scheduling & logging**
- [x] [EPIC-049](docs/issues/epics/EPIC-049-expanded-scripting-environment-dsl-job-scheduling-and-execution-logging.md) Expanded scripting environment, DSL, job scheduling & logging
  - [x] [STORY-0149](docs/issues/stories/STORY-0149-rename-scriptrunner-to-avoid-trademark-conflicts.md) Rename ScriptRunner to ScriptStudio & FrogScript (avoid Adaptavist trademark)
  - [x] [STORY-0150](docs/issues/stories/STORY-0150-in-app-code-execution-console-and-dsl-research.md) In-app code execution console and DSL research
  - [x] [STORY-0151](docs/issues/stories/STORY-0151-scripting-ad-hoc-runs-scheduled-jobs-and-event-driven-triggers.md) Scripting ad-hoc runs, scheduled recurring jobs, and event-driven triggers
  - [x] [STORY-0152](docs/issues/stories/STORY-0152-dedicated-internal-stdout-and-stderr-execution-logging.md) Dedicated internal stdout and stderr execution logging

## v1.0.0 (milestones 14.1 to 14.5, [docs/v1.0.0-roadmap.md](docs/v1.0.0-roadmap.md))
Eight epics and 23 stories, in the roadmap's order. Each item is ticked when its acceptance criteria hold and `scripts/check.sh` passes.

**Milestone 14.1: UX ergonomics and operational planning**
- [x] [EPIC-056](docs/issues/epics/EPIC-056-homework-and-assignment-planner.md) Homework planner
  - [x] [STORY-0165](docs/issues/stories/STORY-0165-homework-domain-model-and-vault-persistence.md) Homework model, storage and TOML form
  - [x] [STORY-0166](docs/issues/stories/STORY-0166-homework-due-dates-and-system-reminders.md) Lead-time reminders and calendar markers
  - [x] [STORY-0167](docs/issues/stories/STORY-0167-homework-management-and-planner-ui.md) Planner screen (tabs, countdowns, create/edit dialog, checklist ticks, delete with confirmation)
- [x] [EPIC-057](docs/issues/epics/EPIC-057-ui-ux-streamlining-tooltips-and-universal-import-hub.md) UI/UX streamlining and Universal Import Hub: STORY-0168, STORY-0169, STORY-0170

**Milestone 14.2: In-app extensibility and memory drills**
- [x] EPIC-058 FrogScript Wiki: STORY-0171, STORY-0172
- [x] [EPIC-059](docs/issues/epics/EPIC-059-memory-mode-rehearsal.md) Memory Mode
  - [x] [STORY-0173](docs/issues/stories/STORY-0173-memory-session-engine-and-pair-mechanics.md) Session engine, deck pairing, flip states
  - [x] STORY-0174 Grid UI and card backs (flip, keyboard, names)
  - [x] STORY-0175 Time attack and records, and the launch from the rehearse setup and the stack page

**Milestone 14.3: Cognitive telemetry and user modeling**
- [x] EPIC-060 Cognitive profiling (done 2026-11-23: the optimizer and the schedule use the calibrated start)
  - [x] STORY-0176 Telemetry schema, recording and queries (step 30)
  - [x] STORY-0177 Difficulty and velocity engine (done 2026-11-23: per-item difficulty list and the adaptive-against-fixed comparison on the Cognitive profile; the D0 start checked in the running app)
  - [x] STORY-0178 Cognitive profile dashboard (curves, distribution and semantic trend, BUG-0030)

**Milestone 14.4: University academic mastery lab**
- [x] EPIC-061 Case-study decision matrices
  - [x] STORY-0179 Case-study model and vault storage (model, validation, question files with the case fields)
  - [x] STORY-0180 Rubric evaluator
  - [x] STORY-0181 Case-study rehearsal UI (vignette, choice and justification, evaluation pills)
  - [x] STORY-0200 Local-model scoring of justifications and the rubric prompt template (small local models unmeasured)
  - [x] STORY-0199 Case studies in the rehearsal run queue (check, suggested rating, score recorded like a typed answer's)
- [x] EPIC-062 Psychometric formula engine (done 2026-11-23: z-score, T score, IQ-scale score, 95% confidence band in full solution, and Hint control)
  - [x] STORY-0182 Formulas, seeded generator, precision tests
  - [x] STORY-0183 Step trace and diagnosis
  - [x] STORY-0184 Math drill view (standard-error formula: working rows, keypad, check, solution)
  - [x] BUG-0051 Praise with a wrong working step (fixed and closed 2026-11-23)
- [x] [EPIC-063](docs/issues/epics/EPIC-063-altklausur-and-mock-exam-simulation-lab.md) Altklausur lab (done 2026-11-23: STORY-0185–0187 and STORY-0207 all done)
  - [x] [STORY-0185](docs/issues/stories/STORY-0185-mock-exam-structure-and-timer-controls.md) Mock exam structure, navigation and timer
  - [x] [STORY-0186](docs/issues/stories/STORY-0186-lecture-slide-driven-exam-generator.md) Exam generation from slides via the local SLM (done 2026-11-23: verified live with Ollama `qwen2.5:1.5b`)
  - [x] [STORY-0187](docs/issues/stories/STORY-0187-exam-grading-and-university-grade-scale.md) Grade scale, summation, the graded review and the pass mark setting
  - [x] [STORY-0207](docs/issues/stories/STORY-0207-exam-generation-item-format-selection-modal.md) Interactive modal to choose item formats during mock exam generation (done 2026-11-23)

**Milestone 14.5: release hardening and 1.0.0 launch**
- [ ] Regression audit, packaging (AppImage, tarball), documentation, release tag `v1.0.0`

## Next: Performance Benchmarking (`EPIC-067`) & Classroom Simulation (`EPIC-068`)

**Performance benchmarking & continuous improvement**
- [ ] [EPIC-067](docs/issues/epics/EPIC-067-performance-benchmarking-and-continuous-improvement.md) Performance benchmarking and continuous improvement (`blocked` on owner's reference machine)
  - [ ] [STORY-0201](docs/issues/stories/STORY-0201-reference-runs-on-the-owners-machine-with-identity-recorded.md) Reference runs on the owner's machine (`blocked`; `TASK-0278` in review, `TASK-0279` interleaved paired runs in backlog)
  - [ ] [STORY-0202](docs/issues/stories/STORY-0202-regression-rule-compare-two-runs-flag-only-real-large-changes.md) Regression rule (`review`; `TASK-0282` release check in backlog)
  - [ ] [STORY-0203](docs/issues/stories/STORY-0203-budgets-replaced-by-measured-baselines.md) Provisional budgets replaced by measured baselines (`blocked`)
  - [ ] [STORY-0204](docs/issues/stories/STORY-0204-frame-time-measured-in-the-running-window.md) Frame time measured in the running window (`backlog`; `TASK-0283`, `TASK-0284`)
  - [ ] [STORY-0205](docs/issues/stories/STORY-0205-allocations-in-render-loops-with-dhat-and-a-profile-of-the-slowest-screen.md) Render-loop allocations with `dhat` and slowest-screen profile (`backlog`; `TASK-0285`, `TASK-0286`)

**Classroom simulation**
- [x] [EPIC-068](docs/issues/epics/EPIC-068-classroom-simulation-with-ai-teacher-and-students.md) Classroom simulation: AI-driven lectures, simulated students, group tasks, and graded submissions (`done`)
  - [x] [STORY-0208](docs/issues/stories/STORY-0208-classroom-session-domain-model-and-local-persistence.md) Classroom session domain model and local storage persistence ([TASK-0287](docs/issues/tasks/TASK-0287-classroom-domain-types-and-state-transitions.md), [TASK-0288](docs/issues/tasks/TASK-0288-sqlite-schema-and-local-classroom-session-store.md)) (done 2026-11-23)
  - [x] [STORY-0209](docs/issues/stories/STORY-0209-ai-teacher-and-simulated-student-lecture-orchestration.md) AI teacher and simulated student lecture orchestration ([TASK-0289](docs/issues/tasks/TASK-0289-lecture-grounding-context-budgeting-and-persona-prompts.md), [TASK-0290](docs/issues/tasks/TASK-0290-classroom-turn-orchestrator-and-service-use-cases.md))
  - [x] [STORY-0210](docs/issues/stories/STORY-0210-individual-and-group-task-assignments-turn-in-and-grading.md) Individual and group task assignments, turn-in workflow, and automated grading ([TASK-0291](docs/issues/tasks/TASK-0291-solo-and-group-task-assignment-and-collaboration.md), [TASK-0292](docs/issues/tasks/TASK-0292-task-turn-in-and-teacher-grading-pipeline.md))
  - [x] [STORY-0211](docs/issues/stories/STORY-0211-classroom-chat-ui-session-browser-and-assignment-workspace.md) Classroom-chat UI, session browser, and assignment submission workspace ([TASK-0293](docs/issues/tasks/TASK-0293-classroom-setup-dialog-and-lecture-chat-view.md), [TASK-0294](docs/issues/tasks/TASK-0294-assignments-panel-group-collaboration-turn-in-and-grades.md), [TASK-0295](docs/issues/tasks/TASK-0295-saved-classroom-sessions-browser-tests-and-control-hooks.md))

## Shipped in v1.2.0 (2026-11-26)
- [x] [BUG-0057](docs/issues/bugs/BUG-0057-home-three-columns-overflow-and-overlap-on-long-titles.md) Home screen three-column cards overflow and overlap adjacent columns on long item titles
- [x] [EPIC-074](docs/issues/epics/EPIC-074-nautilus-style-folders-view-redesign.md) Nautilus/Thunar-style Folders view redesign (view switcher, Icon Grid, Detailed List, 8-way sorting, breadcrumbs, selection status pill)

## Planned: v1.3.0 (UI Polish, Bug Fixes & Usable Match Mode)
- [x] [BUG-0058](docs/issues/bugs/BUG-0058-views-and-modals-missing-redesign-overhaul.md) Redesign overhaul for missing views and modals (Items / All items, Create an item modal, Import and export modal, Profile popover, Vault switcher)
- [x] [BUG-0059](docs/issues/bugs/BUG-0059-item-view-title-headline-editors-and-color-input-cut-off.md) Item view cut-offs (Title input, gradient headline, Front/Back editors, Color `#RRGGBB` input)
- [x] [STORY-0243](docs/issues/stories/STORY-0243-horizontal-scrollability-and-responsive-sizing-for-constrained-widgets.md) Horizontal scrollability and responsive sizing for constrained widgets
- [ ] [EPIC-075](docs/issues/epics/EPIC-075-usable-match-mode-sourced-from-vault-flashcards.md) Make Match mode usable (real vault flashcards/questions, scope picker, playable tile grid)
  - [ ] [STORY-0244](docs/issues/stories/STORY-0244-load-real-vault-pairs-for-match-mode.md) Load real front/back pairs from vault flashcards and questions ([TASK-0300](docs/issues/tasks/TASK-0300-vault-pair-extraction-and-sampling-in-service.md), [TASK-0301](docs/issues/tasks/TASK-0301-wire-match-load-pairs-worker-effect.md))
  - [ ] [STORY-0245](docs/issues/stories/STORY-0245-match-mode-scope-picker-responsive-grid-and-empty-state.md) Match mode scope picker, responsive tile grid, and empty state ([TASK-0302](docs/issues/tasks/TASK-0302-match-scope-selector-pair-count-and-new-round-hud.md), [TASK-0303](docs/issues/tasks/TASK-0303-responsive-match-tile-grid-and-empty-state.md), [TASK-0304](docs/issues/tasks/TASK-0304-headless-ui-and-service-tests-for-match-mode.md))

## Planned: v1.4.0 (Exam Scheduler)
- [ ] [EPIC-076](docs/issues/epics/EPIC-076-exam-scheduler-with-adaptive-preparation-window.md) Exam scheduler: automatically schedule exam-linked stacks and items within a user-set or stats-proposed week window
  - [ ] [STORY-0246](docs/issues/stories/STORY-0246-exam-to-item-linkage-model-and-sqlite-persistence.md) Exam-to-item linkage model and SQLite persistence ([TASK-0305](docs/issues/tasks/TASK-0305-exam-plan-and-prep-window-domain-types.md), [TASK-0306](docs/issues/tasks/TASK-0306-sqlite-migration-and-store-for-exam-plans.md))
  - [ ] [STORY-0247](docs/issues/stories/STORY-0247-adaptive-preparation-window-estimator-and-windowed-scheduler.md) Adaptive preparation-window estimator and windowed item scheduler ([TASK-0307](docs/issues/tasks/TASK-0307-stats-driven-preparation-window-estimator.md), [TASK-0308](docs/issues/tasks/TASK-0308-windowed-item-distribution-and-rebalancing-scheduler.md))
  - [ ] [STORY-0248](docs/issues/stories/STORY-0248-exam-scheduler-ui-preview-and-calendar-rehearsal-integration.md) Exam Scheduler UI, workload preview, and Calendar/Home/Rehearsal integration ([TASK-0309](docs/issues/tasks/TASK-0309-exam-scheduler-dialog-and-workload-preview-ui.md), [TASK-0310](docs/issues/tasks/TASK-0310-surface-exam-blocks-in-calendar-home-and-rehearsal.md), [TASK-0311](docs/issues/tasks/TASK-0311-headless-ui-and-service-tests-for-exam-scheduler.md))
