---
id: EPIC-038
type: epic
title: "CLI control protocol for interactive AI agent application operation"
status: done
parents: []
labels: [m12, core, service, ui, app]
acceptance_criteria:
  - "AI agents can interactively operate a running or headless StudyFrog instance via a structured CLI control protocol with full access to all application features, screens, and actions"
  - "The control protocol exposes live UI state trees, semantic screen snapshots, and real-time console and application logs"
  - "StudyFrog stores application files, configurations, runtime control endpoints, and structured console logs in a standardized ~/.local/ directory hierarchy"
---

## Context
While MCP (`EPIC-037`) exposes vault data and domain tools to LLM clients, autonomous coding and desktop AI agents also need a **CLI Control Protocol** to interactively operate the StudyFrog application itself—navigating screens, triggering any `Msg` or `Service` operation, inspecting current UI/application state, starting and stepping through rehearsal or exam runs, taking screenshots, and streaming live console and diagnostic logs. Pairing this interactive control protocol with a standardized `~/.local/` application directory layout (`STORY-0122`) ensures AI agents and developers have a single predictable place for configurations, logs, and runtime control sockets.

## Description
Deliver a full-featured CLI Control Protocol and `~/.local/` application directory layout for StudyFrog:
1. **Interactive CLI Control Protocol (`STORY-0124`):**
   - Provide a bidirectional CLI control interface (`studyfrog-app ctl <command>` and interactive newline-delimited JSON / REPL mode `studyfrog-app --control-stdio` or Unix domain socket in `~/.local/share/studyfrog/run/control.sock`).
   - Grant AI agents full operational access to all StudyFrog features:
     - **Navigation & UI Driving:** Switch screens (`Home`, `Folders`, `Items`, `Stack`, `Rehearse`, `Calendar`, `Board`, `Reports`, `Diary`, `Achievements`, `Graph`, `Assistant`, `Settings`, `Help`), open/edit/delete items, move stacks, and trigger any Elm-architecture `Msg`.
     - **State & Accessibility Tree Inspection:** Query the current `State` summary, active screen widgets, AccessKit semantic node tree, or capture deterministic PNG screenshots.
     - **Domain & Rehearsal Operation:** Start, answer, flip, rate, pause, or finish rehearsal and exam runs; run imports/exports; query reports and forecasts; manage calendar and board items.
     - **Console & Event Log Streaming:** Tail or query structured console logs, notices, background worker events, and error traces in real time (`ctl logs --follow --json`).
2. **Standardized `~/.local/` Application Files, Logs & Configurations (`STORY-0122`):**
   - Consolidate StudyFrog's application-level files under `~/.local/share/studyfrog/` (or XDG-compliant `~/.local/` subdirectories) containing global configurations (`app.toml`, `secrets.toml`, `profiles/`), structured rotating application and console logs (`logs/studyfrog.log`, `logs/console.jsonl`), cached SLM models, and runtime control socket/PID metadata.

## Acceptance Criteria
- [x] AI agents can interactively operate a running or headless StudyFrog instance via a structured CLI control protocol with full access to all application features, screens, and actions
- [x] The control protocol exposes live UI state trees, semantic screen snapshots, and real-time console and application logs
- [x] StudyFrog stores application files, configurations, runtime control endpoints, and structured console logs in a standardized ~/.local/ directory hierarchy

## Design Notes
Child stories:
- STORY-0122: StudyFrog application files in ~/.local/ with logs and configurations (estimate: 5)
- STORY-0124: Interactive CLI control protocol and console log streaming for AI agents (estimate: 8)

Architecture guardrails:
- Because StudyFrog's UI already follows a pure Elm architecture (`State` + `Msg` -> `(State, Vec<Effect>)` in `studyfrog-ui`), the CLI control protocol can serialize/deserialize commands directly into `Msg` or `Service` calls and serialize `State` snapshots and console ring-buffer logs back to JSON without duplicating any business logic!
- Strictly adhere to P1 (zero inherent `impl` blocks) and P2 (≤ 250 lines per file).

## Progress (2026-11-08)
Both stories are in review: STORY-0122 (application home and logs) and STORY-0124 (the control protocol: `studyfrog --control-stdio`, `run/control.sock` served by the window and by headless instances, `studyfrog ctl`, `logs --follow`). Not yet: full coverage of every screen's actions (see STORY-0124's Progress), no AccessKit tree.

## Verification
Review (2026-11-22): criterion 1 NOT MET: calendar, board, diary, reports, achievements, settings, assistant and the folder operations are reached by `nav` and show only the screen name; no AccessKit tree; no stack delete (see STORY-0124). Criterion 2 NOT MET: `ctl state` is a semantic snapshot, but there is no live UI widget/state tree; live logs are MET (`ctl logs --follow` streamed a notice and a control warning as they happened). Criterion 3 MET: with no `STUDYFROG_HOME`, a headless run under a fake `HOME` created `~/.local/share/studyfrog/run/` and `config/app.toml` (`appearance density` write); with `STUDYFROG_HOME` set, `config/`, `logs/studyfrog.log`, `logs/console.jsonl` and `run/control.sock`/`control.json` were written.

Review (2026-11-23): criterion 1 NOT MET, narrowed: calendar, board, diary and reports can now be driven (`board`, `board add`, `diary list|open|add`, `reports <tab>`, `calendar view|period` answer with their data; see STORY-0124's note). Still open: achievements, settings, assistant, graph, match, folders and PDF are reachable by `nav` only, and there is no stack delete. Criterion 2 NOT MET: no live UI widget or AccessKit tree; `state` is a semantic snapshot, and live logs are still MET (`ctl logs --follow`, 2026-11-22). Criterion 3 MET (unchanged).

Review (2026-11-23, closing the epic): criterion 1 MET for a running window and for every command family: the seven screens that had only `nav` (achievements, graph, assistant, settings, match, folders, PDF open) and `stack delete` now have commands, and `ui tree`, `ui find` and `ui click` drive any widget of the window (the window's AccessKit tree, captured by a plugin when asked). Criterion 2 MET: the tree is the live UI state (`ui tree`), `state` the semantic snapshot, and logs stream (`ctl logs --follow`). Limits: a headless instance has no widgets (`ui` is refused with the reason); the graph has no stack scope; a configured assistant was not run. Details and the answers are in STORY-0124's Verification.

## Dependencies
EPIC-008 (UI Shell), EPIC-028 (Extensibility), and EPIC-037 (MCP).

## Open Questions
None.
