---
id: EPIC-041
type: epic
title: "Modularity: system-driven application architecture and decoupled background services"
status: backlog
parents: []
labels: [m12, core, service, ui]
acceptance_criteria:
  - "Core domain systems (file surveys, vault indexing, background reindexing, achievements, notifications, and analytics calculation) are wired into application lifecycle and background workers rather than driven lazily by specific UI views"
  - "Switching vaults or loading the dashboard immediately dispatches file surveys and background indexing so all files and items appear without requiring the user to open the Files screen first"
  - "UI screens interact with core systems through decoupled query and command functions, separating domain system execution from screen render loops"
---

## Context
In StudyFrog, several background subsystems, data loaders, and surveys are currently driven lazily by individual UI view draw functions rather than by centralized application systems.

For example, file surveys only execute once the user explicitly navigates to and opens the Files view (`view/library_pdfs.rs` / `view/folders`). When a user switches to a vault or starts up the application on the Home dashboard or Library table, the file survey is not actioned. As a result, existing files (such as newly dropped PDF documents, markdown notes, or external additions) do not appear in summaries or indices until the files view is opened.

The application architecture needs a system-driven redesign where core domain processes operate independently as services wired into application lifecycle hooks and background worker pipelines. UI views then become pure observers and consumers of these systems rather than the triggers for their execution.

## Description
Redesign StudyFrog's architecture toward system-driven modularity:

1. **System Lifecycle Hooks & Vault Initialization (`studyfrog-service`, `studyfrog-ui`):**
   - Wire file discovery, file survey, and full-text index updates into vault lifecycle hooks (`open_vault`, `switch_vault`, `fs_poll`).
   - Ensure file surveys and document cataloging run immediately upon vault attachment, populating store state so the dashboard, library, and search display all existing files immediately.
2. **Decoupled Background Services (`studyfrog-service::poll`, `crates/studyfrog-ui/src/exec.rs`):**
   - Establish background worker routines for:
     - File system polling and survey execution (`poll_files`).
     - Vault health check and broken link verification.
     - Achievement evaluation and notification generation.
     - Analytics and familiarity cache warming.
3. **Pure Screen Presenters (`crates/studyfrog-ui/src/view/`):**
   - Refactor views (such as `view/library_pdfs.rs`, `view/folders`, `view/reports.rs`) so they do not initiate asynchronous surveys during egui `show` calls.
   - Views receive snapshot state via `State` and issue explicit user commands (`Msg`) rather than driving background data gathering.

## Acceptance Criteria
- [ ] Core domain systems (file surveys, vault indexing, background reindexing, achievements, notifications, and analytics calculation) are wired into application lifecycle and background workers rather than driven lazily by specific UI views
- [ ] Switching vaults or loading the dashboard immediately dispatches file surveys and background indexing so all files and items appear without requiring the user to open the Files screen first
- [ ] UI screens interact with core systems through decoupled query and command functions, separating domain system execution from screen render loops

## Design Notes
Touchpoints across crates:
- `crates/studyfrog-service`: `poll.rs`, `scan.rs`, `pdf_index.rs`, `session.rs`.
- `crates/studyfrog-ui`: `exec.rs` (worker task dispatch), `update.rs` (lifecycle event orchestration), `view/library_pdfs.rs` (removing lazy survey dispatch).
- Architectural constraints: strictly adhere to P1 (zero inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- EPIC-006 (Storage).
- EPIC-008 (UI shell).
- EPIC-018 (Service layer completion).

## Open Questions
- None.
