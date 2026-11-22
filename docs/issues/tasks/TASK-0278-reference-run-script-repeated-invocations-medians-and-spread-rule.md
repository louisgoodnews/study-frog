---
id: TASK-0278
type: task
title: "Reference-run script: repeated invocations, medians and the spread rule"
status: review
epic: EPIC-067
parents: ['STORY-0201']
estimate: 2
labels: [m14, perf, tooling]
acceptance_criteria:
  - 'scripts/bench-reference.sh runs each budgeted case N times (default 5), one process per invocation, and writes each median (ns) to <out>/<case>.ns'
  - 'A case whose runs spread more than 5% is extended to 10 runs'
  - 'Run here for one case (fsrs_schedule_100k) it wrote its runs and extended them to 10, with the machine identity in machine.txt'
  - 'Run on the owner''s machine for every budgeted case, with the saved baselines (owner)'
---

## Context
Part of STORY-0201. The script covers the budgeted cases of BENCHMARKING.md.

## Description
`scripts/bench-reference.sh <label> <out> [runs] [case-slug]`. It builds each bench once, then runs criterion
with `--exact <case id>` and `--save-baseline <label>-run<i>` per invocation. The median is read from
criterion's `time:` line on stdout (the same line `scripts/bench-spread.sh` reads), so case names with spaces and
commas work. Change lines (`change: time: [-1.2% ...]`) are skipped.

## Acceptance Criteria
- [x] Each budgeted case runs N times (default 5) and each median goes to `<out>/<case>.ns`
- [x] A case whose runs spread more than 5% is extended to 10 runs
- [x] One case run here: 5 runs spread 9.8%, extended to 10; machine.txt written
- [ ] Run on the owner's machine for every budgeted case, with the saved baselines

## Design Notes
- Not done: interleaving of two builds (TASK-0279). This script repeats one build; the paired comparison needs TASK-0279.
- The container used here is not a reference machine. The 9.8% spread there is the container's noise, not a result.
- Measured here, for the record: fsrs_schedule_100k median over 10 runs 4.23 ms (criterion's default sample count per run).

## Dependencies
STORY-0201. TASK-0279 for the paired comparison. The owner's machine for the final run (blocks STORY-0203).

## Open Questions
- Whether 27 cases x 5 to 10 invocations fits a schedule on the owner's machine. Estimate: about 20 to 40 minutes.
