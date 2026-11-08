---
id: EPIC-028
type: epic
title: Extensibility, plugin modding, and Python API
status: in_progress
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
- [ ] StudyFrog exposes a documented Python API allowing scripts to interact with vault items, metadata, and scheduling without compiling Rust code
- [ ] Application functionality can be extended via modding plugins or scripts executed as cron jobs, event-bound hooks, or via a built-in console
- [ ] External AI models (Google Gemini and Deepseek) and optimized PDF viewing integrate cleanly under version 0.5.0 planning

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

## Dependencies
Phase M11 (0.4.0) completed.

## Open Questions
1. What sandbox boundaries and filesystem permissions should be presented to the user when installing third-party Python plugins?
