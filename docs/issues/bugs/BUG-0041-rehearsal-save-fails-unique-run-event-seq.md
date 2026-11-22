---
id: BUG-0041
type: bug
title: "Rehearsal shows 'Could not save your progress' (UNIQUE constraint on run_event run_uid, seq)"
status: backlog
estimate: 3
labels: [m14, rehearse, data]
acceptance_criteria:
  - "Starting, flipping, rating and stopping rehearsal runs, including a run started while a finished run is still on screen, saves every step without a database error"
  - "A run_event sequence number is never written twice for the same run"
---

## Context
Found while auditing STORY-0169 and STORY-0187 in the running app (2026-11-22). Three times in one session the status bar showed "Could not save your progress: database error: UNIQUE constraint failed: run_event.run_uid, run_event.seq", and the log had "RecordStep failed" before it. The sequence was: a finished answer-mode run on screen, then "rehearse setup" and "rehearse start" for a new run, "rehearse flip", "rehearse rate good", then "rehearse stop". I could not reproduce it on a clean sequence afterwards (flip, rate, stop on a fresh run, and rating after a finished run, all saved), so the trigger is not isolated.

Screenshot: `/tmp/claude-0/audit/g5/0186-setup.png` (the error in the status bar, with a memory game on screen after the stop).

## Description
Find how a run's event sequence can repeat: the next sequence number is probably taken before the previous event is written, or a new run reuses the previous run's counter. Add a test that starts a run while a finished run is held, and fix the numbering. Check the log after each step.

## Acceptance Criteria
- [ ] Starting, flipping, rating and stopping rehearsal runs, including a run started while a finished run is still on screen, saves every step without a database error
- [ ] A run_event sequence number is never written twice for the same run

## Design Notes
- Touchpoints: `crates/studyfrog-store` (run_event writes), `crates/studyfrog-service` (RecordStep), `crates/studyfrog-ui/src/exec/rehearse*`.
- Guardrails: P1, P6.

## Dependencies
- STORY-0187

## Open Questions
- The exact trigger (a run started while a finished run is on screen) is not confirmed.
