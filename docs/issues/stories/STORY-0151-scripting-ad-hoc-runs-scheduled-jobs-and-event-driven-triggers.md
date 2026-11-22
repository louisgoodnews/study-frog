---
id: STORY-0151
type: story
title: "Scripting ad-hoc runs, scheduled recurring jobs, and event-driven triggers"
status: done
epic: EPIC-049
parents: [EPIC-049]
estimate: 5
labels: [m12, ext, console]
acceptance_criteria:
  - "The in-app scripting environment allows running ad-hoc scripts on demand with parameter inputs"
  - "Users can define, edit, enable, and disable scheduled cron jobs directly from the application interface"
  - "Users can bind scripts to application lifecycle events (vault open/close, card creation, rehearsal completion) with visual status indicators"
---

## Context
Automating study routines requires multiple execution patterns: executing a quick ad-hoc transformation script, scheduling recurring background jobs (such as daily backups, weekly retention reports, or morning deck preparation), and triggering scripts automatically in response to application lifecycle events (e.g. tagging newly created cards, logging notes upon rehearsal completion). Users need a unified interface in StudyFrog to configure, test, and monitor these automation modes.

## Description
Deliver comprehensive script execution management within StudyFrog:
1. **Ad-Hoc Execution:** Run any saved or newly drafted script with customizable runtime arguments and immediate execution tracing.
2. **Scheduled Cron Jobs:** UI manager for `<app_home>/scripts/scripts.toml` cron definitions (`@daily`, `@weekly`, `0 8 * * mon-fri`) with next-run calculation and manual trigger overrides.
3. **Event-Driven Triggers:** Visual binding interface for application lifecycle events (`vault_opened`, `vault_closed`, `item_created`, `item_saved`, `rehearsal_finished`) with enable/disable toggles and execution history.

## Acceptance Criteria
- [x] The in-app scripting environment allows running ad-hoc scripts on demand with parameter inputs
- [x] Users can define, edit, enable, and disable scheduled cron jobs directly from the application interface
- [x] Users can bind scripts to application lifecycle events (vault open/close, card creation, rehearsal completion) with visual status indicators

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/view/console.rs`, `crates/studyfrog-service/src/scripting/config.rs`, `crates/studyfrog-service/src/scripting/cron.rs`.
- Guardrails: P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- EPIC-049 (Expanded scripting environment, DSL, job scheduling, and execution logging).
- STORY-0149 (Rename ScriptRunner to avoid trademark conflicts).

## Window check (2026-11-22)

Checked in the running window: the job editor lists jobs from `scripts.toml` with their next run and enabled state, reports a bad file with the TOML error, and "Run now" runs the script; its output reaches the console as "started" and "finished" lines. Not checked: adding a job from the form and saving, and event-bound jobs firing.

## Open Questions
- Conflict resolution when multiple event-bound scripts trigger simultaneously on card creation.

## Implementation (2026-11-22, audit)

Audit (2026-11-22): criterion 1 MET: the job editor's "Run a script by hand" took the arguments "xray yankee"; the output and the start and finish lines appear in the console (/tmp/claude-0/audit/g5/0151-adhoc-run.png, /tmp/claude-0/audit/g5/0151-console-run.png). The name shown for a by-hand run is wrong (BUG-0042). Criterion 2 MET: a job was added from the form (script hello.py, schedule "0 9 * * mon-fri", event vault_closed), the schedule and events were edited, the enabled tick on weekly-off was set, and Save jobs wrote scripts.toml (/tmp/claude-0/audit/g5/0151-form-saved2.png; the file's form-job and weekly-off entries). Criterion 3 MET: jobs bound to item_created, vault_closed, vault_opened and rehearsal_finished each ran and printed their output (/tmp/claude-0/audit/g5/0151-event-run2.png, /tmp/claude-0/audit/g5/0151-jobs-events.png; logs show each run). The list shows "on schedule", "on event" and "off" labels. Defect: script stderr lands in the stdout stream (BUG-0033).
