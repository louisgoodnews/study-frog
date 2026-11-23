---
id: BUG-0041
type: bug
title: "Rehearsal shows 'Could not save your progress' (UNIQUE constraint on run_event run_uid, seq)"
status: done
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
- [x] Starting, flipping, rating and stopping rehearsal runs, including a run started while a finished run is still on screen, saves every step without a database error
- [x] A run_event sequence number is never written twice for the same run

## Design Notes
- Touchpoints: `crates/studyfrog-store` (run_event writes), `crates/studyfrog-service` (RecordStep), `crates/studyfrog-ui/src/exec/rehearse*`.
- Guardrails: P1, P6.

## Dependencies
- STORY-0187

## Open Questions
- The exact trigger (a run started while a finished run is on screen) is not confirmed.

## Fix

- Cause: the sequence number of each run event was counted by the rehearsal session in memory (`next_seq`, which starts at 0 for each session). Opening the vault again runs `recover_runs`, which appends `run_aborted` at the next free number in the database. A run still on screen across that reopen then wrote its next event with a number that was already stored, which the UNIQUE constraint rejects.
- Fix: `studyfrog-store::runs::append_events_numbered` gives each event the next number of its run's stored log, inside the step's transaction (`record_step` uses it). The session's number is only an order hint now.
- Tests: `crates/studyfrog-service/tests/run_seq.rs`: `a_step_after_a_reopen_does_not_repeat_a_sequence_number` (fails on the old code with `UNIQUE constraint failed: run_event.run_uid, run_event.seq`, passes now) and `quick_successive_ratings_are_all_saved_with_unique_numbers`.
- App evidence: the pre-fix binary (`/tmp/claude-0/audit-bin/studyfrog`) reported `Could not save your progress: database error: UNIQUE constraint failed: run_event.run_uid, run_event.seq` after a vault reopen during a run; the fixed build saved the same steps with no notice, and rapid ratings showed no notice either.
- Not reproduced: the exact trigger in the report (a finished run on screen, then setup and start). The UI ignores a start while a run is on screen, so no second run is written in that case; the reopen during a live run is the reproduced path.
