# Roadmap

Phases M0 to M8. Each phase has entry criteria, an epic breakdown (every epic has at least one story, every story at least one task) and **exit criteria**: what must be demonstrably working. Issue files for a phase are created when it becomes next (M0 seeded now); until then the tree below is the plan of record.

| Phase | Name | Epics | Detail |
|---|---|---|---|
| M0 | Bootstrap | EPIC-001, EPIC-002 | [M0-bootstrap](M0-bootstrap.md) |
| M1 | Core domain | EPIC-003, EPIC-004 | [M1-core-domain](M1-core-domain.md) |
| M2 | Markdown and storage | EPIC-005, EPIC-006 | [M2-markdown-and-storage](M2-markdown-and-storage.md) |
| M3 | Rehearsal engine | EPIC-007 | [M3-rehearsal-engine](M3-rehearsal-engine.md) |
| M4 | UI shell | EPIC-008, EPIC-009 | [M4-ui-shell](M4-ui-shell.md) |
| M5 | Calendar and board | EPIC-010, EPIC-011 | [M5-calendar-and-board](M5-calendar-and-board.md) |
| M6 | SLM comparison | EPIC-012, EPIC-013 | [M6-slm-comparison](M6-slm-comparison.md) |
| M7 | Achievements and reporting | EPIC-014, EPIC-015 | [M7-achievements-and-reporting](M7-achievements-and-reporting.md) |
| M8 | Extensibility and plugin boundaries | EPIC-016, EPIC-017 | [M8-extensibility-and-plugin-boundaries](M8-extensibility-and-plugin-boundaries.md) |

## M0 Bootstrap

Workspace, guardrails and documentation exist; nothing else.

**Entry:** Repository exists with an initial commit.

**Exit criteria:**
- `cargo check --workspace` passes with all 12 crates
- `scripts/check.sh` runs fmt, clippy, test, line-count and exits 0 on the tree
- `scripts/check.sh` fails when a `.rs` file exceeds 250 lines (demonstrated)
- Dependency-direction check fails on a forbidden edge (demonstrated)
- All docs in the required tree exist; ADRs 0001-0005 written
- M0 issues seeded in `docs/issues/`

**Work breakdown:**
- **EPIC-001** Workspace and guardrails
  - **STORY-0001** Workspace skeleton
    - TASK-0001 Convert to Cargo workspace
    - TASK-0002 Create crate skeletons with doc comments
    - TASK-0003 Dependency-direction check script
  - **STORY-0002** Guardrail scripts
    - TASK-0004 scripts/check.sh
    - TASK-0005 scripts/linecount.sh
    - TASK-0006 impl-block scan for P1
- **EPIC-002** Documentation library
  - **STORY-0003** Core docs and ADRs
    - TASK-0007 Vision, architecture, principles, glossary
    - TASK-0008 ADRs 0001-0005
  - **STORY-0004** Roadmap and issue tracking
    - TASK-0009 Roadmap and phase files
    - TASK-0010 Issue format and README
    - TASK-0011 Specs, research, contributing guides

## M1 Core domain

Pure data types and the scheduling algorithm.

**Entry:** M0 exit met.

**Exit criteria:**
- `studyfrog-core` defines ids, Stack, Note, Card, Question, Review, Run and event types, data only
- FSRS and SM-2 pass proptests and golden vectors
- `cargo bench -p studyfrog-srs` baseline recorded
- No `impl` behaviour blocks (scan passes)

**Work breakdown:**
- **EPIC-003** Core domain types
  - **STORY-0005** Identity and content types
    - TASK-0012 Id newtypes and arena indices
    - TASK-0013 Stack, Note, Card, Question structs
  - **STORY-0006** Review and run data
    - TASK-0014 Review and Rating types
    - TASK-0015 Run and event types
- **EPIC-004** SRS scheduling
  - **STORY-0007** FSRS implementation
    - TASK-0016 FSRS pure functions
    - TASK-0017 FSRS property and golden tests
  - **STORY-0008** SM-2 baseline
    - TASK-0018 SM-2 functions
    - TASK-0019 SRS criterion benchmark

## M2 Markdown and storage

Vault files parse into core types and are indexed in SQLite.

**Entry:** M1 exit met.

**Exit criteria:**
- A fixture vault of 10k notes scans, parses and indexes under budget
- `parse(render(x)) == x` property holds for all kinds
- Renaming a file keeps history (id-based)
- Corrupt files are reported, not fatal
- Migrations apply on an empty DB and on the previous version

**Work breakdown:**
- **EPIC-005** Markdown and front-matter
  - **STORY-0009** Front-matter parse and render
    - TASK-0020 Schema structs and validation
    - TASK-0021 Fence splitter and serde-saphyr parsing
    - TASK-0022 Round-trip property test
  - **STORY-0010** Body parsing
    - TASK-0023 Wikilink extraction
    - TASK-0024 Card and question section split
- **EPIC-006** Storage
  - **STORY-0011** SQLite schema and migrations
    - TASK-0025 Initial schema
    - TASK-0026 Migration runner
  - **STORY-0012** File layer and index sync
    - TASK-0027 Vault scanner
    - TASK-0028 Incremental reindex
    - TASK-0029 File watcher

## M3 Rehearsal engine

Headless rehearsal runs with latency capture and SRS updates.

**Entry:** M2 exit met.

**Exit criteria:**
- A scripted run (test harness) produces the expected event log
- Latency excludes paused time (tested)
- Run results update SRS state and persist
- Crash recovery marks runs aborted
- Rehearsal step bench under budget

**Work breakdown:**
- **EPIC-007** Rehearsal engine
  - **STORY-0013** Run state machine
    - TASK-0030 Run input types and transition errors
    - TASK-0031 Transition functions
  - **STORY-0014** Latency capture
    - TASK-0032 Injected monotonic clock
    - TASK-0033 Persist run events
  - **STORY-0015** Queue selection
    - TASK-0034 Due-first queue builder
    - TASK-0035 Rehearsal benchmark

## M4 UI shell

First usable desktop app: browse, edit, rehearse.

**Entry:** M3 exit met; ADR 0002 accepted.

**Exit criteria:**
- App launches and opens a vault
- User can create a stack, write a card, rehearse it with the keyboard, and see it rescheduled
- Markdown editor with preview works
- Link graph renders 5k nodes at 60 fps with no per-frame allocation
- `update` layer has unit tests; UI crate holds no business logic

**Work breakdown:**
- **EPIC-008** UI shell
  - **STORY-0016** App frame
    - TASK-0036 eframe window and navigation
    - TASK-0037 Elm-style update loop and effects
  - **STORY-0017** Stack browser and rehearsal screen
    - TASK-0038 Stack browser
    - TASK-0039 Rehearsal screen
- **EPIC-009** Editor and graph
  - **STORY-0018** Markdown editor and preview
    - TASK-0040 Editor widget
    - TASK-0041 Preview renderer
  - **STORY-0019** Link graph view
    - TASK-0042 Force-directed layout (SoA)
    - TASK-0043 Graph render benchmark

## M5 Calendar and board

Planning and task management.

**Entry:** M4 exit met.

**Exit criteria:**
- Sessions can be scheduled, linked to stacks, and recur
- Board enforces WIP limits (property-tested)
- Drag and push semantics work in the UI
- Calendar and board persist across restarts

**Work breakdown:**
- **EPIC-010** Calendar and planning
  - **STORY-0020** Session planning
    - TASK-0044 Session model and recurrence
    - TASK-0045 Calendar view
- **EPIC-011** Scrum/kanban board
  - **STORY-0021** Board model
    - TASK-0046 Columns and WIP limits
    - TASK-0047 Move and push semantics
  - **STORY-0022** Board UI
    - TASK-0048 Board view with drag and drop

## M6 SLM comparison

Local SLM scoring of free-text answers.

**Entry:** M4 exit met (M5 not required); ADR 0004 spike run.

**Exit criteria:**
- ADR 0004 Accepted with measured numbers
- User types an answer and receives a score and verdict offline
- Scoring proptests pass; p95 latency within budget
- Runtime swappable behind `SlmRuntime` (second impl or test double proves it)

**Work breakdown:**
- **EPIC-012** SLM runtime
  - **STORY-0023** Runtime trait and implementation
    - TASK-0049 SlmRuntime trait
    - TASK-0050 candle embedder and spike
    - TASK-0051 Model management
- **EPIC-013** Comparison scoring
  - **STORY-0024** Scoring contract
    - TASK-0052 Score functions
    - TASK-0053 Scoring property tests
  - **STORY-0025** UI integration
    - TASK-0054 Answer input in rehearsal
    - TASK-0055 SLM benchmark

## M7 Achievements and reporting

Motivation and insight.

**Entry:** M3 and M4 exit met.

**Exit criteria:**
- Stack and global achievements unlock from run data
- Retention curve, latency distribution and review-load forecast are shown
- Forecast backtest error reported on real or simulated history
- Reports compute within budget on 100k reviews

**Work breakdown:**
- **EPIC-014** Achievements
  - **STORY-0026** Rule engine
    - TASK-0056 Rule data model
    - TASK-0057 Evaluator
  - **STORY-0027** Achievement UI
    - TASK-0058 Achievements screen and toasts
- **EPIC-015** Reporting and forecasting
  - **STORY-0028** Retention and latency reports
    - TASK-0059 Retention curves
    - TASK-0060 Latency distributions
  - **STORY-0029** Forecasting
    - TASK-0061 Review-load forecast
    - TASK-0062 Report screens

## M8 Extensibility and plugin boundaries

Explicit extension points and release readiness.

**Entry:** M1-M7 exit met.

**Exit criteria:**
- Documented extension traits exist for card types and importers
- An example extension is built without modifying core crates
- Profiling pass done; all benchmarks within budget
- Linux package (AppImage or Flatpak) builds from CI

**Work breakdown:**
- **EPIC-016** Plugin boundaries
  - **STORY-0030** Extension traits
    - TASK-0063 Define extension points
    - TASK-0064 Registry as plain data
  - **STORY-0031** Example extension
    - TASK-0065 Example card type or importer
- **EPIC-017** Release hardening
  - **STORY-0032** Performance and packaging
    - TASK-0066 Profiling pass
    - TASK-0067 Linux packaging
