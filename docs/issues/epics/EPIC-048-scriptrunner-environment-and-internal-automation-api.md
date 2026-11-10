---
id: EPIC-048
type: epic
title: Scriptrunner environment and internal application automation API
status: done
parents: []
labels: [m12, ext, console]
acceptance_criteria:
  - "StudyFrog provides a full scriptrunner environment with interactive Python execution and internal automation APIs"
  - "Learners and power users can test, automate, and extend application workflows via scripting directly within the UI"
  - "The scripting environment provides safe execution boundaries, output logging, and hot-reloading"
---

## Context
While StudyFrog has CLI scripting (`studyfrog script run`) and control protocol sockets (`run/control.sock`), power users and developers need a rich, interactive in-app scriptrunner environment. This environment should provide an internal API for querying items, automating deck creation, executing custom macros, and prototyping new application capabilities directly inside the app.

## Description
Expand the Console into a complete in-app ScriptRunner development environment:
1. Provide an interactive scripting notebook/console with syntax highlighting, stdout/stderr streaming, and REPL history.
2. Expose an ergonomic internal Python API (`studyfrog.app`) allowing scripts to query the active vault, schedule reviews, manipulate timetable entries, and register custom text macros.
3. Support saving script snippets to `<vault>/.studyfrog/scripts/` with one-click runner buttons.

## Acceptance Criteria
- [x] StudyFrog provides a full scriptrunner environment with interactive Python execution and internal automation APIs
- [x] Learners and power users can test, automate, and extend application workflows via scripting directly within the UI
- [x] The scripting environment provides safe execution boundaries, output logging, and hot-reloading

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/view/console.rs`, `crates/studyfrog-ext/`, `docs/specs/scripting.md`.
- Child story: STORY-0148.
- Guardrails: P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- EPIC-028 (Extensibility, plugin modding, and Python API).

## Open Questions
- None.
