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

- **EPIC-019** Rehearsal insights (added during M6)
  - **STORY-0035** Marks, reading speed and the post-run review
    - TASK-0081 Difficulty marks during a run
    - TASK-0082 Reading-speed rating suggestion
    - TASK-0083 Post-run review and applying a suggestion
    - TASK-0084 Reading-speed test
    - TASK-0085 Slow reads suggest difficulty

## Exit criteria

- [x] ADR 0004 Accepted with measured numbers
- [x] User types an answer and receives a score and verdict offline
- [x] Scoring proptests pass; p95 latency within budget (lexical runtime: about 21 µs)
- [x] Runtime swappable behind `SlmRuntime` (second impl or test double proves it)

## Out of scope

Everything listed under later phases in [ROADMAP](ROADMAP.md).

## Risks

See ADRs and `STATUS.md` for open decisions affecting this phase.
