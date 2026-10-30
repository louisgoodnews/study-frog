# M6: SLM comparison

**Goal:** Local SLM scoring of free-text answers.

**Entry criteria:** M4 exit met (M5 not required); ADR 0004 spike run.

## Epics, stories, tasks

- **EPIC-012** SLM runtime
  - **STORY-0023** Runtime trait and implementation
    - TASK-0049 SlmRuntime trait
    - TASK-0050 candle embedder and spike
    - TASK-0051 Model management
- **EPIC-013** Comparison scoring
  - **STORY-0024** Scoring contract
    - TASK-0052 Score functions
    - TASK-0053 Scoring property tests
  - **STORY-0025** UI integration
    - TASK-0054 Answer input in rehearsal
    - TASK-0055 SLM benchmark

## Exit criteria

- [ ] ADR 0004 Accepted with measured numbers
- [ ] User types an answer and receives a score and verdict offline
- [ ] Scoring proptests pass; p95 latency within budget
- [ ] Runtime swappable behind `SlmRuntime` (second impl or test double proves it)

## Out of scope

Everything listed under later phases in [ROADMAP](ROADMAP.md).

## Risks

See ADRs and `STATUS.md` for open decisions affecting this phase.
