---
id: STORY-0122
type: story
title: StudyFrog application files in ~/.local/ with logs and configurations
status: review
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
- [ ] StudyFrog stores its application-level configurations, profiles, runtime state, and logs under a unified ~/.local/ directory hierarchy (~/.local/share/studyfrog/)
- [ ] All application notices, worker events, warnings, and errors are written to rotating plain-text and JSON Lines log files (logs/studyfrog.log and logs/console.jsonl)
- [ ] Users and CLI agents can inspect the ~/.local/ paths and tail or export console logs via CLI flags or the Settings screen

## Design Notes
- Touchpoints:
  - `crates/studyfrog-config/src/`: path resolution (`studyfrog_home_dir()`) and migration helper.
  - `crates/studyfrog-service/src/logging.rs`: structured JSONL and text log writer with rotation.
  - `crates/studyfrog-app/src/`: wire startup/runtime logging and `--print-paths` / `logs` CLI subcommands.
- Strictly enforce P1 (zero inherent `impl` blocks) and P2 (≤ 250 lines per file).

## Progress (2026-11-08)
Built and tested: `studyfrog-config::paths` (home from `STUDYFROG_HOME`, `XDG_DATA_HOME` or `~/.local/share/studyfrog`), `studyfrog-store::app_migrate` (the old config directory is copied in, never moved or deleted, an existing file is never overwritten, `secrets.toml` ends up `0600`), `studyfrog-service::logging` (text and JSON Lines, 5 MB x 3 files), every notice written to both logs through `NotifyEffect::Log`, config, models and logs read from the new home, `studyfrog --print-paths`, `studyfrog logs [--tail N] [--follow] [--json]` (tried against a real binary), and an "Application files" section in Settings > Data with copy buttons.

Open: only notices are logged, not other worker events, import/export diagnostics or control-protocol events (the protocol is STORY-0124); `profiles/` and `run/` are named but nothing writes to them yet (no socket exists); no "Open logs folder" button (it needs an effect that starts the file manager); `profiles.toml` stays in `config/`; the Settings section was not looked at in a real window; the migration ran in tests only, not against a real `~/.config/studyfrog`.

## Dependencies
EPIC-038.

## Open Questions
None.
