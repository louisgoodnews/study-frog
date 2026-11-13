---
id: EPIC-049
type: epic
title: "Expanded scripting environment, DSL, job scheduling, and execution logging"
status: backlog
parents: []
labels: [m12, ext, console]
acceptance_criteria:
  - "The in-app scripting environment is rebranded and renamed from ScriptRunner to a distinct, proprietary trademark-safe name (e.g. FrogScript / ScriptStudio) avoiding conflicts with Adaptavist"
  - "Users can execute code directly from a built-in scripting console with support for research into a domain-specific language (DSL)"
  - "Scripting provides full support for ad-hoc execution, scheduled cron jobs, and event-driven trigger bindings directly configurable from the UI"
  - "StudyFrog implements dedicated internal stdout and stderr log capturing and display streams for user-authored scripts"
---

## Context
Following the prototype of the in-app scripting console in milestone 0.7.0, power users and developers need an expanded, production-grade automation workbench. To respect external trademarks (specifically Adaptavist's registered "ScriptRunner" trademark), the feature must be rebranded. Furthermore, scripting capabilities must expand beyond basic REPL input to include rich in-app console execution, potential Domain-Specific Language (DSL) integration, scheduled cron and event-driven automation management, and dedicated stdout/stderr diagnostic streams.

## Description
Deliver the expanded in-app scripting studio and execution infrastructure:
1. **Rebranding & Trademark Safety (`STORY-0149`):** Rename ScriptRunner to a unique, distinctive trademark-safe brand (e.g., FrogScript or ScriptStudio) across UI, code, and documentation.
2. **In-App Console & DSL Research (`STORY-0150`):** Provide interactive multi-line script execution in the UI and conduct deep research into embedding a lightweight DSL (such as Rhai or a custom FrogScript DSL) alongside Python.
3. **Execution Modes: Ad-Hoc, Scheduled, and Event-Driven (`STORY-0151`):** Support interactive runs, recurring cron schedule definitions (`@daily`, `@weekly`), and event hook subscriptions (`vault_opened`, `item_created`, `rehearsal_finished`) with in-app management.
4. **Dedicated Internal stdout and stderr Logging (`STORY-0152`):** Implement isolated stream capture, multiplexing, timestamps, and styled presentation separating standard application output from error diagnostics.

## Acceptance Criteria
- [ ] The in-app scripting environment is rebranded and renamed from ScriptRunner to a distinct, proprietary trademark-safe name (e.g. FrogScript / ScriptStudio) avoiding conflicts with Adaptavist
- [ ] Users can execute code directly from a built-in scripting console with support for research into a domain-specific language (DSL)
- [ ] Scripting provides full support for ad-hoc execution, scheduled cron jobs, and event-driven trigger bindings directly configurable from the UI
- [ ] StudyFrog implements dedicated internal stdout and stderr log capturing and display streams for user-authored scripts

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/view/console.rs`, `crates/studyfrog-ext/`, `crates/studyfrog-service/src/scripting/`, `docs/specs/scripting.md`, `docs/research/`.
- Child stories: STORY-0149, STORY-0150, STORY-0151, STORY-0152.
- Guardrails: P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- EPIC-048 (Scriptrunner environment and internal application automation API).
- EPIC-028 (Extensibility, plugin modding, and Python API).

## Open Questions
- Target DSL candidate evaluation (Rhai embedded Rust script engine vs. standard Python sub-process IPC).
