---
id: STORY-0148
type: story
title: Console expansion to full scriptrunner environment with internal API
status: done
epic: EPIC-048
parents: [EPIC-048]
estimate: 5
labels: [m12, ext, console, research]
acceptance_criteria:
  - "The Console screen provides a multi-line script editor with run, pause, stop, and clear controls"
  - "Output log stream captures stdout, stderr, and returned objects with formatted timestamps"
  - "The internal Python API provides methods for vault querying, card creation, calendar events, and custom macro registration"
---

## Context
Power users want to use StudyFrog as a programmable study workbench. Expanding the Console into an integrated ScriptRunner enables automating repetitive tasks, building bespoke deck generators, and executing automated timetable syncs without having to switch to an external terminal.

## Description
Deliver the in-app ScriptRunner and internal API:
1. In `crates/studyfrog-ui/src/view/console.rs`, build an editor panel with execution toolbar (Run Script, Stop, Save, Clear Console).
2. Wire real-time communication between the UI console and the underlying Python worker or sub-process runner via the control protocol.
3. Author the internal API specification and user guide in `docs/research/scriptrunner-and-internal-api.md`.

## Acceptance Criteria
- [x] The Console screen provides a multi-line script editor with run, pause, stop, and clear controls
- [x] Output log stream captures stdout, stderr, and returned objects with formatted timestamps
- [x] The internal Python API provides methods for vault querying, card creation, calendar events, and custom macro registration

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/view/console.rs`, `crates/studyfrog-ext/`, `crates/studyfrog-ui/src/console_link.rs`.
- Guardrails: P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- EPIC-048 (Scriptrunner environment and internal application automation API).

## Open Questions
- None.
