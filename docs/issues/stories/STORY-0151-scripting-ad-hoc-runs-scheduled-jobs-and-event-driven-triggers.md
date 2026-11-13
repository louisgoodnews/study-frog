---
id: STORY-0151
type: story
title: "Scripting ad-hoc runs, scheduled recurring jobs, and event-driven triggers"
status: backlog
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
- [ ] The in-app scripting environment allows running ad-hoc scripts on demand with parameter inputs
- [ ] Users can define, edit, enable, and disable scheduled cron jobs directly from the application interface
- [ ] Users can bind scripts to application lifecycle events (vault open/close, card creation, rehearsal completion) with visual status indicators

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/view/console.rs`, `crates/studyfrog-service/src/scripting/config.rs`, `crates/studyfrog-service/src/scripting/cron.rs`.
- Guardrails: P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- EPIC-049 (Expanded scripting environment, DSL, job scheduling, and execution logging).
- STORY-0149 (Rename ScriptRunner to avoid trademark conflicts).

## Open Questions
- Conflict resolution when multiple event-bound scripts trigger simultaneously on card creation.
