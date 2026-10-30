# M1: Core domain

**Goal:** Pure data types and the scheduling algorithm.

**Entry criteria:** M0 exit met.

## Epics, stories, tasks

- **EPIC-003** Core domain types
  - **STORY-0005** Identity and content types
    - TASK-0012 Id newtypes and arena indices
    - TASK-0013 Stack, Note, Card, Question structs
  - **STORY-0006** Review and run data
    - TASK-0014 Review and Rating types
    - TASK-0015 Run and event types
- **EPIC-004** SRS scheduling
  - **STORY-0007** FSRS implementation
    - TASK-0016 FSRS pure functions
    - TASK-0017 FSRS property and golden tests
  - **STORY-0008** SM-2 baseline
    - TASK-0018 SM-2 functions
    - TASK-0019 SRS criterion benchmark

## Exit criteria

- [ ] `studyfrog-core` defines ids, Stack, Note, Card, Question, Review, Run and event types, data only
- [ ] FSRS and SM-2 pass proptests and golden vectors
- [ ] `cargo bench -p studyfrog-srs` baseline recorded
- [ ] No `impl` behaviour blocks (scan passes)

## Out of scope

Everything listed under later phases in [ROADMAP](ROADMAP.md).

## Risks

See ADRs and `STATUS.md` for open decisions affecting this phase.
