---
id: EPIC-028
type: epic
title: "Extensibility, plugin modding, and Python API"
status: done
parents: []
labels: [m12, ext, slm]
acceptance_criteria:
  - "StudyFrog exposes a documented Python API allowing scripts to interact with vault items, metadata, and scheduling without compiling Rust code"
  - "Application functionality can be extended via modding plugins or scripts executed as cron jobs, event-bound hooks, or via a built-in console"
  - "External AI models (Google Gemini and Deepseek) and optimized PDF viewing integrate cleanly under version 0.5.0 planning"
---

## Context
Following the 0.4.0 release (Phase M11), StudyFrog requires deeper extensibility and external intelligence capabilities planned for version 0.5.0 (Phase M12). Users need to customize and extend StudyFrog through modding and scripts written in Python, run scheduled or event-driven tasks, interact via an in-app console, and connect modern external AI models with high-performance PDF viewing.

## Description
Provide an extensibility and modding framework for StudyFrog planned for version 0.5.0:
1. Support plugin modding to expand application functionality (importers, custom cards, automated views).
2. Enable script execution across three key modes:
   - Cron jobs (scheduled periodic tasks, such as daily vault health checks or queue exports).
   - Event-bound triggers (reacting to application lifecycle events, such as vault opening, item creation, or rehearsal completion).
   - Built-in interactive console (providing a live environment inside the application for executing commands and ad-hoc scripts).
3. Expose a Python API for StudyFrog so users can write Python scripts that run right away, avoiding the need to recompile the Rust codebase.
4. House version 0.5.0 feature stories: external model API connectivity for Google Gemini (STORY-0083) and Deepseek (STORY-0084), PDF viewer performance optimization (STORY-0085), and PDF viewer illegibility bug fix (BUG-0010).

## Acceptance Criteria
- [x] StudyFrog exposes a documented Python API allowing scripts to interact with vault items, metadata, and scheduling without compiling Rust code
- [x] Application functionality can be extended via modding plugins or scripts executed as cron jobs, event-bound hooks, or via a built-in console
- [x] External AI models (Google Gemini and Deepseek) and optimized PDF viewing integrate cleanly under version 0.5.0 planning

## Design Notes
Child issues planned for version 0.5.0:
- STORY-0083: Google Gemini in models for StudyFrog
- STORY-0084: Deepseek in models for StudyFrog
- STORY-0085: Performance optimization in StudyFrog pdf-viewer
- BUG-0010: PDF pages are illegible depending on zoom or page fit mode

Architecture and boundaries:
- Crate `studyfrog-ext` currently provides compiled-in function pointer registries for card types and format importers (ADR 0008).
- Subprocess JSON-RPC Architecture: To preserve workspace safety rule P9 (`unsafe_code = "forbid"`) and prevent cross-platform packaging hazards with Linux dynamic Python runtimes, Python scripts execute in an out-of-process runner communicating over standard streams (stdin/stdout) via JSON-RPC.
- Scripting interfaces: Define a lifecycle event dispatcher in `studyfrog-service` (`on_item_saved`, `on_rehearsal_completed`, `on_cron_tick`), an in-app interactive console view in `studyfrog-ui`, and a cron scheduler daemon in the service layer.
- Release target: version 0.5.0 (Phase M12).

Scripting part (decided in ADR 0013, `docs/adr/0013-scripting-out-of-process.md`): STORY-0130 lifecycle events and
`subscribe`; STORY-0131 the Python client `python/studyfrog`; STORY-0132 the script runner (`scripts.toml`, cron,
event-bound jobs, `studyfrog script ...`); STORY-0133 the Console screen; STORY-0134 docs, examples and script
plugins. The bridge is the existing control protocol rather than a new JSON-RPC protocol, and the lifecycle events
are derived in the UI layer (where messages are applied), not in `studyfrog-service`.

## Progress (2026-11-08)
The scripting part is done (STORY-0130 to STORY-0134, each verified by tests against the real binary; see the
stories). "Scheduling" in the first criterion means the rehearsal steps and the schedule an item shows; the control
protocol has no command that edits an item's review dates directly. Plugin modding is honest about its reach: scripts
can import, generate cards and automate notes and exports, but cannot add entries to the window's dialogs (compiled-in
extensions only, ADR 0008). The epic stays in progress because STORY-0085 (PDF performance) is in progress and
STORY-0083/STORY-0084 are in review (another package); the third criterion depends on them.

## Dependencies
Phase M11 (0.4.0) completed.

## Open Questions
1. What sandbox boundaries and filesystem permissions should be presented to the user when installing third-party Python plugins?
