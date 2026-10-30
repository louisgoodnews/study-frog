# M3: Rehearsal engine

**Goal:** Headless rehearsal runs with latency capture and SRS updates.

**Entry criteria:** M2 exit met.

## Epics, stories, tasks

- **EPIC-007** Rehearsal engine
  - **STORY-0013** Run state machine
    - TASK-0030 Run input types and transition errors
    - TASK-0031 Transition functions
  - **STORY-0014** Latency capture
    - TASK-0032 Injected monotonic clock
    - TASK-0033 Persist run events
  - **STORY-0015** Queue selection
    - TASK-0034 Due-first queue builder
    - TASK-0035 Rehearsal benchmark

## Exit criteria

- [ ] A scripted run (test harness) produces the expected event log
- [ ] Latency excludes paused time (tested)
- [ ] Run results update SRS state and persist
- [ ] Crash recovery marks runs aborted
- [ ] Rehearsal step bench under budget

## Out of scope

Everything listed under later phases in [ROADMAP](ROADMAP.md).

## Risks

See ADRs and `STATUS.md` for open decisions affecting this phase.
