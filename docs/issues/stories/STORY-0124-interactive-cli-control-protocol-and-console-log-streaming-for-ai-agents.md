---
id: STORY-0124
type: story
title: Interactive CLI control protocol and console log streaming for AI agents
status: backlog
epic: EPIC-038
parents: [EPIC-038]
estimate: 8
labels: [m12, service, ui, app]
acceptance_criteria:
  - "AI agents can operate StudyFrog interactively via studyfrog-app ctl <command> or a persistent JSON-RPC/JSONL control stream (--control-stdio or ~/.local/share/studyfrog/run/control.sock)"
  - "The control protocol provides full coverage of application features: screen navigation, UI widget/state inspection, item/stack CRUD, rehearsal and exam run stepping, query execution, imports/exports, and screenshots"
  - "Agents can read and live-stream structured console logs, notices, and error traces via the control protocol"
---

## Context
To let AI agents interactively operate StudyFrog with full access to every application feature and console log (`EPIC-038`), StudyFrog needs a deterministic, scriptable CLI control protocol. Because StudyFrog's UI is built on a pure Elm-style architecture (`State` + `Msg` -> `(State, Vec<Effect>)`), the control protocol can inspect the live `State`, dispatch any UI `Msg` or `studyfrog-service` action, wait for background `Effect` completion, and stream console logs from `~/.local/share/studyfrog/logs/console.jsonl` (`STORY-0122`).

## Description
Implement the interactive CLI Control Protocol across `studyfrog-ui`, `studyfrog-service`, and `studyfrog-app`:
1. **Control Transport Modes (`crates/studyfrog-app`):**
   - **Live Socket Attach (`studyfrog-app ctl ...`):** Connects to a running desktop or headless StudyFrog instance via `~/.local/share/studyfrog/run/control.sock` to inspect or drive the live window in real time.
   - **Headless Interactive Stream (`studyfrog-app --control-stdio [--vault <path>]`):** Runs StudyFrog headlessly, reading JSON control commands from `stdin` and emitting structured JSON state/event/log responses on `stdout`.
2. **Full Application Feature Coverage:**
   - `state`: Return a structured JSON snapshot of the active screen, open vault, selected stack/item, active rehearsal/exam prompt, modal state, and recent notices.
   - `nav <screen>`: Navigate to any screen (`home`, `folders`, `items`, `rehearse`, `calendar`, `board`, `reports`, `diary`, `achievements`, `graph`, `assistant`, `settings`, `help`) or `back` / `forward`.
   - `item <create|open|edit|delete|color|move>` and `stack <create|open|move|settings>`: Full item and stack operations.
   - `rehearse <setup|start|flip|answer|rate|mark|skip|pause|stop>`: Interactively step through rehearsal and exam runs.
   - `query <expr>`: Execute any StudyFrog query or search and return matching items.
   - `screenshot <path.png>`: Render the current frame to a PNG file.
   - `logs [--tail N] [--follow] [--level info|warn|error]`: Return or stream structured console log entries (`console.jsonl`).

## Acceptance Criteria
- [ ] AI agents can operate StudyFrog interactively via studyfrog-app ctl <command> or a persistent JSON-RPC/JSONL control stream (--control-stdio or ~/.local/share/studyfrog/run/control.sock)
- [ ] The control protocol provides full coverage of application features: screen navigation, UI widget/state inspection, item/stack CRUD, rehearsal and exam run stepping, query execution, imports/exports, and screenshots
- [ ] Agents can read and live-stream structured console logs, notices, and error traces via the control protocol

## Design Notes
- Touchpoints:
  - `crates/studyfrog-ui/src/control.rs`: pure command parser and `State` -> JSON summary serializer.
  - `crates/studyfrog-app/src/control_server.rs`: Unix socket and `--control-stdio` loop.
- Strictly adhere to P1 (zero inherent `impl` blocks) and P2 (≤ 250 lines per file).

## Dependencies
EPIC-038, STORY-0122.

## Open Questions
None.
