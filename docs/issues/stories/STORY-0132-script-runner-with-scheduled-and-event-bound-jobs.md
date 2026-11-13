---
id: STORY-0132
type: story
title: "Script runner with scheduled and event-bound jobs"
status: done
epic: EPIC-028
parents: [EPIC-028]
estimate: 8
labels: [m12, service, ui, app]
acceptance_criteria:
  - "`<app home>/scripts/scripts.toml` lists jobs (script, args, cron schedule and/or event kinds, timeout, access); invalid entries are reported with their reason and left out; tested"
  - "Cron schedules (five fields, lists, ranges, steps, @hourly/@daily/@weekly) are parsed in-house and matched against local time; due jobs are found with an injected clock, each minute once; unit tests never sleep"
  - "A job runs as a child process with a timeout (killed when it runs over), its output and errors go to the application log, and it gets a control socket of its own limited to its access (read by default, write, or destructive only with allow_destructive)"
  - "Every window and headless instance runs the scheduled and event-bound jobs; `studyfrog script run FILE`, `script list`, `script daemon` and `script install-client` work; tested with the real binary (an event-bound Python script triggered by item creation)"
---
## Context
EPIC-028 asks for cron jobs and event-bound triggers (ADR 0013).

## Description
Pure parts in `studyfrog-service/src/scripting/` (config, cron, triggers, process execution); the runner thread and the scoped sockets in `studyfrog-ui`; the CLI in `studyfrog-ui/src/script_cli.rs`. Tasks: TASK-0247, TASK-0248, TASK-0249.

## Acceptance Criteria
- [x] `<app home>/scripts/scripts.toml` lists jobs (script, args, cron schedule and/or event kinds, timeout, access); invalid entries are reported with their reason and left out; tested
- [x] Cron schedules (five fields, lists, ranges, steps, @hourly/@daily/@weekly) are parsed in-house and matched against local time; due jobs are found with an injected clock, each minute once; unit tests never sleep
- [x] A job runs as a child process with a timeout (killed when it runs over), its output and errors go to the application log, and it gets a control socket of its own limited to its access (read by default, write, or destructive only with allow_destructive)
- [x] Every window and headless instance runs the scheduled and event-bound jobs; `studyfrog script run FILE`, `script list`, `script daemon` and `script install-client` work; tested with the real binary (an event-bound Python script triggered by item creation)

## Design Notes
ADR 0013 (`docs/adr/0013-scripting-out-of-process.md`). Build vs buy: a five-field cron matcher is about 120 lines; the `cron` crate brings a chrono-based iterator, seconds and years we do not want, so it is written in-house.

## Progress (2026-11-08)
Built: `studyfrog-service/src/scripting/` (`config.rs` scripts.toml with per-entry errors, `cron.rs` five-field cron with names, steps and shorthands, `due.rs` ticker and event matching, `exec.rs` child process with timeout, process-group kill and capped output, `log.rs` log entries); in `studyfrog-ui`: `script_runner.rs` (scheduler and event threads, one run per job at a time), `script_launch.rs` (command line, environment `STUDYFROG_SOCKET/HOME/JOB/ACCESS/EVENT/BIN`, `PYTHONPATH`), `script_socket.rs` (a per-run socket served under the job's policy, or a refusing proxy to another instance), `script_client.rs` (the Python package embedded and installed into `scripts/lib/`), `script_cli.rs`; `Job::RunScoped` carries a script's policy into the engine and the window.

Verified: `crates/studyfrog-service/tests/scripting_cron.rs`, `scripting_config.rs`, `scripting_exec.rs` (schedules, an injected clock with catch-up and a clock going back, invalid entries, timeouts that kill what the script started, output caps); `crates/studyfrog-ui/tests/script_runner.rs` (scheduler and event steps with an injected time, no double start, environment and policies); `crates/studyfrog-app/tests/script_binary.rs` (real binary: an `item_created` Python job reads over its read-only socket and is refused a write, its output is logged, its socket removed; an `allow_destructive` job deletes an item; `script list`, `install-client`) and `script_run_binary.rs` (`script run` with a headless instance started for it, and through a proxy to a running one with read and write access; a timeout). By hand in a real window (2026-11-08): the window ran an event-bound job on `vault_opened` and on an item created with `studyfrog ctl`, and logged both.

Not covered: the scheduler thread with the real clock is only exercised through `scheduler_step` (no test waits for a real minute); scheduled jobs only run while an instance runs (window, headless or `script daemon`); `script run` through a running instance cannot exceed that instance's policy (destructive needs it started with `--control-allow-destructive`); `script daemon` ended by a signal leaves `run/control.sock` until the next start replaces it; Unix only.

## Dependencies
STORY-0130, STORY-0131.

## Open Questions
None.
