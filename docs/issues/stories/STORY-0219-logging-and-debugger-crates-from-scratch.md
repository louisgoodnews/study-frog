---
id: STORY-0219
type: story
title: "In-house logging and debugger crates (studyfrog-log, studyfrog-debug), written from scratch"
status: review
epic: EPIC-071
parents: ['EPIC-071']
estimate: 5
labels: [m14, tooling, core]
acceptance_criteria:
  - "studyfrog-log provides levels, a process-wide minimum level and sink set at start-up, a ring of the 512 most recent records, level parsing, a one-line record format, a size-rotating file sink and the log_error! to log_trace! macros, with no external crate"
  - "studyfrog-debug provides timers, checkpoints, value dumps, checks, named counters and debug_dump!, built on studyfrog-log, with no external crate"
  - "Both crates have no inherent impl blocks, no file over 250 lines, and are in the allowed table of scripts/check-deps.sh (log: no studyfrog dependency; debug: log only)"
  - "Unit tests cover level ordering and filtering, the ring cap, record quoting, check and counts; the owner runs them"
---

## Context
EPIC-071 wants StudyFrog to own its logging and debugging helpers. The application already writes a rotating log
(STORY-0122), but the logging API the rest of the code would call is not a crate of its own, and the debugging helpers
(timers, dumps, checks, counters) do not exist. Both are small and specific to this project, so they are written here
rather than taken from a crate. This story writes the two crates only; wiring them into the application is a later
story.

## Description
- `crates/studyfrog-log`: `Level` (Error to Trace, ordered so that `record <= minimum` means enabled), `name`,
  `parse_level`, `Record`, `emit`, the process-wide `set_level` / `level` and `set_sink` (default: one line per record to
  stderr), `recent(n)` over a ring of 512 records, `format_line`, `file_sink(path, max_bytes, keep)` for a rotating file,
  and the `log_*!` macros with the form `log_info!(target, "format", args; key = value)`.
- `crates/studyfrog-debug`: `timed` (a `Timer` that logs `elapsed_us` at Debug when dropped), `elapsed_us`, `checkpoint`
  (Trace), `dump` (Debug, the `{:?}` text cut to 2000 characters), `check` (logs at Error and returns the condition),
  `count` / `counts` (a session counter registry sorted by name) and `debug_dump!`.
- Tests in each crate's `tests/` folder. They are written, not run: the owner runs them.

## Acceptance Criteria
- [ ] studyfrog-log API as described above, with no external crate
- [ ] studyfrog-debug API as described above, built on studyfrog-log
- [ ] No inherent `impl` block; every file at most 250 lines; `scripts/check-deps.sh` allows the two crates' dependencies
- [ ] Unit tests for level ordering and filtering, the ring cap, record quoting, `check` and `counts` written

## Design Notes
- Build versus buy (P10): `log`, `tracing` and similar crates were considered. They solve a wider problem (subscribers,
  spans, many formats) than StudyFrog needs, and each adds a dependency that the no-external-crate direction for these
  two crates rules out. The needs here are a level, a ring, one line per record and a rotating file, which is about 300
  lines. Built in-house.
- The minimum level is an `AtomicU8`, so a disabled record costs one atomic load and one comparison. The sink and the
  ring are behind locks and are touched only for records that are kept.
- The macros check the level before they format anything, so arguments of disabled records are not evaluated.
- `emit` puts a record into the ring before the sink runs, and the ring keeps the record even when the sink fails.
- The file sink ignores write errors on purpose: a log that stops the program is worse than a missing line.
- The `Timer` logs on drop; the caller binds it to a named variable.
- Global state (level, sink, ring, counters) is process-wide, so the tests that change it run in one test function per
  file.
- Clippy forbids `std::thread::sleep` in this workspace (`clippy.toml`), so the timer test busy-waits.

## Dependencies
- EPIC-071 (this epic).
- STORY-0122 (the existing rotating log). It is not replaced by this story; a later story may move the application onto
  `studyfrog-log`.
- `scripts/check-deps.sh`, `scripts/check-impls.sh`, `scripts/linecount.sh` (gates).

## Open Questions
- Should the application's rotating log (STORY-0122) move onto `studyfrog-log`'s file sink? Owner decision, later story.
- Should `studyfrog-log` gain a JSON Lines sink for `logs/console.jsonl`? Not in this story; the record type has the
  fields it would need.

## Verification note
Written 2026-11-25 without building: no `cargo build`, `check`, `test`, `clippy` or compiler was run. Testing is pending;
the owner runs `scripts/check.sh` and the crate tests. `scripts/board.sh` has not been run, so `BOARD.md` is not
regenerated.
