# M3: Rehearsal engine

**Goal:** Headless rehearsal runs with latency capture and SRS updates.

**Entry criteria:** M2 exit met.

## Epics, stories, tasks

- **EPIC-007** Rehearsal engine
  - **STORY-0013** Run state machine
    - TASK-0030 Run input types and transition errors
    - TASK-0031 Transition functions
    - TASK-0071 Requeue on Again and schedule effect
  - **STORY-0014** Latency capture
    - TASK-0032 Injected monotonic clock
    - TASK-0033 Persist run events
  - **STORY-0015** Queue selection
    - TASK-0034 Spaced (due-first) queue with daily budgets
    - TASK-0035 Rehearsal benchmark
  - **STORY-0033** Run plans, orders and filters
    - TASK-0068 Run plan types in core
    - TASK-0069 Practice orders
    - TASK-0070 Queue filters
    - TASK-0072 Persist run plan (schema v2)
- **EPIC-018** Service layer completion (M3 follow-up)
  - **STORY-0034** Stack settings, SM-2, budgets and filter persistence
    - TASK-0074 Persist the run filter
    - TASK-0075 Stack settings in the index
    - TASK-0076 SM-2 in the service
    - TASK-0077 Per-stack daily budgets

## Exit criteria

- [x] A scripted run (test harness) produces the expected event log
- [x] Latency excludes paused time (tested)
- [x] Run results update SRS state and persist
- [x] Crash recovery marks runs aborted
- [x] Rehearsal step bench under budget
- [x] Runs can be built with any order (due-first, sequential, shuffled, hardest/easiest, stalest, lowest recall) and filters; a practice-only run leaves the schedule untouched

## Out of scope

Everything listed under later phases in [ROADMAP](ROADMAP.md).

## Risks

See ADRs and `STATUS.md` for open decisions affecting this phase.
