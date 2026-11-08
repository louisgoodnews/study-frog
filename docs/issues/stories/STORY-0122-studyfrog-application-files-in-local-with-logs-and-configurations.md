---
id: STORY-0122
type: story
title: StudyFrog application files in ~/.local/ with logs and configurations
status: done
epic: EPIC-038
parents: [EPIC-038]
estimate: 5
labels: [m12, config, service, app]
acceptance_criteria:
  - "StudyFrog stores its application-level configurations, profiles, runtime state, and logs under a unified ~/.local/ directory hierarchy (~/.local/share/studyfrog/)"
  - "All application notices, worker events, warnings, and errors are written to rotating plain-text and JSON Lines log files (logs/studyfrog.log and logs/console.jsonl)"
  - "Users and CLI agents can inspect the ~/.local/ paths and tail or export console logs via CLI flags or the Settings screen"
---

## Context
Currently, StudyFrog scatters global configuration files across XDG config/data paths and holds notices only in an in-memory UI list (`State.notifications`) without persisting structured application or console logs to disk. Centralizing StudyFrog's application files under `~/.local/` (`~/.local/share/studyfrog/`) with persistent, rotating console/diagnostic logs (`logs/studyfrog.log` and `logs/console.jsonl`), configuration files (`config/app.toml`, `config/secrets.toml` with `0600` permissions), user profiles, and runtime sockets makes troubleshooting, self-hosting, and AI agent inspection straightforward.

## Description
Standardize StudyFrog's application directory layout in `~/.local/` and implement persistent console/application logging:
1. **Unified `~/.local/` Application Directory Layout (`crates/studyfrog-config`, `crates/studyfrog-service`):**
   - Root directory: `~/.local/share/studyfrog/` (overridable via `STUDYFROG_HOME` environment variable for isolated testing):
     - `config/app.toml`: Global application settings, recent vaults, appearance, and reading speed thresholds.
     - `config/secrets.toml`: Provider API keys stored with strict `0600` Unix permissions.
     - `profiles/`: User profile configurations and metadata.
     - `logs/studyfrog.log`: Human-readable timestamped application log (with size-based rotation, e.g., 5 MB × 3 files).
     - `logs/console.jsonl`: Structured JSON Lines log of every UI notice, service command, background worker event, import/export diagnostic, and error for CLI and AI agent consumption.
     - `models/`: Downloaded local embedding / SLM models.
     - `run/`: Runtime control socket (`control.sock`) and lock/PID metadata for active instances.
   - Automatically migrate existing `app.toml` / `secrets.toml` files from legacy config paths on startup without losing user settings.
2. **Structured Console & Diagnostic Logger:**
   - Append every `Notice`, service error, vault reindex summary, and control-protocol event to an in-memory ring buffer and flush to `logs/console.jsonl` and `logs/studyfrog.log`.
3. **UI & CLI Access:**
   - Show the `~/.local/share/studyfrog/` paths and a `"Open logs folder"` / `"View console log"` action in Settings.
   - Support `studyfrog-app --print-paths` and `studyfrog-app logs [--tail N] [--follow]`.

## Acceptance Criteria
- [x] StudyFrog stores its application-level configurations, profiles, runtime state, and logs under a unified ~/.local/ directory hierarchy (~/.local/share/studyfrog/)
- [x] All application notices, worker events, warnings, and errors are written to rotating plain-text and JSON Lines log files (logs/studyfrog.log and logs/console.jsonl)
- [x] Users and CLI agents can inspect the ~/.local/ paths and tail or export console logs via CLI flags or the Settings screen

## Design Notes
- Touchpoints:
  - `crates/studyfrog-config/src/`: path resolution (`studyfrog_home_dir()`) and migration helper.
  - `crates/studyfrog-service/src/logging.rs`: structured JSONL and text log writer with rotation.
  - `crates/studyfrog-app/src/`: wire startup/runtime logging and `--print-paths` / `logs` CLI subcommands.
- Strictly enforce P1 (zero inherent `impl` blocks) and P2 (≤ 250 lines per file).

## Progress (2026-11-08)
Built and tested: `studyfrog-config::paths` (home from `STUDYFROG_HOME`, `XDG_DATA_HOME` or `~/.local/share/studyfrog`), `studyfrog-store::app_migrate` (the old config directory is copied in, never moved or deleted, an existing file is never overwritten, `secrets.toml` ends up `0600`), `studyfrog-service::logging` (text and JSON Lines, 5 MB x 3 files), every notice written to both logs through `NotifyEffect::Log`, config, models and logs read from the new home, `studyfrog --print-paths`, `studyfrog logs [--tail N] [--follow] [--json]` (tried against a real binary), and an "Application files" section in Settings > Data with copy buttons.

Since STORY-0124 (2026-11-08): `run/` is used — a running window or headless instance serves `run/control.sock` and names itself in `run/control.json` (stale sockets are replaced) — and control-protocol events are logged: notices raised by control commands go to both logs, and refused or failed commands are written as warnings with the source `control`.

Added on 2026-11-08: **worker events** are logged with the source `worker` by the window's worker thread and by headless instances (`exec_log::execute_logged`, lines chosen by the pure `worker_log.rs`): every effect answered with an error (opening a vault or a PDF, rendering, saving, loading, reports, imports, exports, ...) as a warning naming the effect (`Pdf.Open failed: ...`), import outcomes with skipped rows and preview warnings, export paths, and effects that took two seconds or more (at most 20 lines per effect). **Open logs folder** in Settings > Data starts the system's file manager (`xdg-open`, `open` or `explorer`) through an opener the binary injects (`Host::opener`, `NotifyEffect::OpenLogs`), creating the folder if needed and saying how it went. In Settings > Data the Copy buttons now come before the paths so a long path cannot push them off screen. Help page Settings > Data updated.

Verified: `crates/studyfrog-ui/tests/logs_folder.rs` (the button asks the worker; the worker uses the injected opener, makes the folder, reports success or the opener's error, and says when no opener exists; a failed `OpenVault` and a CSV export land in both log files as `worker` lines), unit tests in `worker_log.rs`; in the real binary `--screen "pdf:bio/missing.pdf"` wrote `WARN worker: Pdf.Open failed: cannot read the file: ...` to `logs/studyfrog.log` and `console.jsonl`, and `--screen settings` (Data tab) shows the Application files section with Open logs folder. The real file manager was not started (it would open on the owner's desktop); the owner's machine shows the migration ran for real (`~/.config/studyfrog/app.toml` kept, a copy in `~/.local/share/studyfrog/config/`).

Earlier open items, now: other worker events and import/export diagnostics are logged; the button exists; `profiles.toml` stays in `config/` inside the home (the criterion asks for the hierarchy, which `config/` is part of); `profiles/` is still named but unused.

Previously open (before 2026-11-08): other worker events and import/export diagnostics are not logged yet (only notices and control-protocol events are); `profiles/` is named but nothing writes to it yet; no "Open logs folder" button (it needs an effect that starts the file manager); `profiles.toml` stays in `config/`; the Settings section was not looked at in a real window; the migration ran in tests only, not against a real `~/.config/studyfrog`.

## Dependencies
EPIC-038.

## Open Questions
None.
