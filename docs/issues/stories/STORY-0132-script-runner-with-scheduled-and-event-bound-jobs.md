---
id: STORY-0132
type: story
title: Script runner with scheduled and event-bound jobs
status: backlog
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
- [ ] `<app home>/scripts/scripts.toml` lists jobs (script, args, cron schedule and/or event kinds, timeout, access); invalid entries are reported with their reason and left out; tested
- [ ] Cron schedules (five fields, lists, ranges, steps, @hourly/@daily/@weekly) are parsed in-house and matched against local time; due jobs are found with an injected clock, each minute once; unit tests never sleep
- [ ] A job runs as a child process with a timeout (killed when it runs over), its output and errors go to the application log, and it gets a control socket of its own limited to its access (read by default, write, or destructive only with allow_destructive)
- [ ] Every window and headless instance runs the scheduled and event-bound jobs; `studyfrog script run FILE`, `script list`, `script daemon` and `script install-client` work; tested with the real binary (an event-bound Python script triggered by item creation)

## Design Notes
ADR 0013 (`docs/adr/0013-scripting-out-of-process.md`). Build vs buy: a five-field cron matcher is about 120 lines; the `cron` crate brings a chrono-based iterator, seconds and years we do not want, so it is written in-house.

## Dependencies
STORY-0130, STORY-0131.

## Open Questions
None.
