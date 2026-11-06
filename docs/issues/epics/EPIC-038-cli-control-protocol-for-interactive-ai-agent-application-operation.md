---
id: EPIC-038
type: epic
title: CLI control protocol for interactive AI agent application operation
status: backlog
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
- [ ] AI agents can interactively operate a running or headless StudyFrog instance via a structured CLI control protocol with full access to all application features, screens, and actions
- [ ] The control protocol exposes live UI state trees, semantic screen snapshots, and real-time console and application logs
- [ ] StudyFrog stores application files, configurations, runtime control endpoints, and structured console logs in a standardized ~/.local/ directory hierarchy

## Design Notes
Child stories:
- STORY-0122: StudyFrog application files in ~/.local/ with logs and configurations (estimate: 5)
- STORY-0124: Interactive CLI control protocol and console log streaming for AI agents (estimate: 8)

Architecture guardrails:
- Because StudyFrog's UI already follows a pure Elm architecture (`State` + `Msg` -> `(State, Vec<Effect>)` in `studyfrog-ui`), the CLI control protocol can serialize/deserialize commands directly into `Msg` or `Service` calls and serialize `State` snapshots and console ring-buffer logs back to JSON without duplicating any business logic!
- Strictly adhere to P1 (zero inherent `impl` blocks) and P2 (≤ 250 lines per file).

## Dependencies
EPIC-008 (UI Shell), EPIC-028 (Extensibility), and EPIC-037 (MCP).

## Open Questions
None.
