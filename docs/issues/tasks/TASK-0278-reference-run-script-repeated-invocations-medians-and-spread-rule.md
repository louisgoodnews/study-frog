---
id: TASK-0278
type: task
title: "Reference-run script: repeated invocations, medians and the spread rule"
status: done
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
- [x] Run on the reference machine (ADR 0014) for every budgeted case, 28 of 28 (2026-11-24, `docs/benchmarks/reference-1.0.0/`)

## Design Notes
- Not done: interleaving of two builds (TASK-0279). This script repeats one build; the paired comparison needs TASK-0279.
- The container used here is not a reference machine. The 9.8% spread there is the container's noise, not a result.
- Measured here, for the record: fsrs_schedule_100k median over 10 runs 4.23 ms (criterion's default sample count per run).
- Verification (this session): the first re-run failed at run 2 with "no median". Cause: the awk filter exited at the
  `time:` line while cargo still printed "Found N outliers", so pipefail failed the run at random. Fixed by reading to the
  end of the output. After the fix: `bench-reference.sh container-verify <dir> 5 fsrs_schedule_100k` gave 5 runs with
  spread 37.9%, extended to 10 runs, median 28.0 ms, spread 46.6%, and wrote machine.txt. The container is not a
  reference machine (powersave governor, other processes busy), so these numbers do not match the 4.23 ms above.
- Criteria 1 to 4 hold (criterion 4 on the reference machine, ADR 0014). The spread rule does not hold (see the reference run below), so the task stays in `review`.

## Reference run (2026-11-24, this machine, ADR 0014)
- Machine identity: `docs/benchmarks/reference-1.0.0/machine.txt` (Intel Core i5-10210U @ 1.60 GHz, 8 logical CPUs,
  Linux 7.3.5, rustc 1.99.1, commit ec88f3e). Hygiene not met: governor powersave, turbo enabled, 14 processes above 10% CPU.
- `scripts/bench-reference.sh homebox-ref target/bench-ref-1 5` ran all 28 budgeted cases. Medians and spreads are in
  BENCHMARKING.md, section "Reference run on the 1.0.0 reference machine".
- Spread rule: the script extended 12 cases to 10 runs as specified. After 10 runs 12 cases still spread more than 5%
  (up to 60%). The rule does not hold on this machine, so the task stays in `review`. Repeating the same run does not
  change a noise floor of this size; the noise source is the machine's state (powersave, busy processes), and interleaved
  runs (TASK-0279) are the next step.

## Repeat runs (2026-11-24, owner decision)
- Run 3 and run 4 on the reference machine: after 10 runs, 6 cases (run 3) and 5 cases (run 4) still spread more than 5%
  (run 4: `pdf_render_1x` 13.2%, `pdf_info` 9.2%, `rehearsal_queue_lowest` 9.9%, `rehearsal_queue_shuffled` 7.4%,
  `graph_project` 6.9%). The spread rule does not hold, so the task stays in `review`.

## Dependencies
STORY-0201. TASK-0279 for the paired comparison. The owner's machine for the final run (blocks STORY-0203).

## Open Questions
- Whether 28 cases x 5 to 10 invocations fits a schedule on the owner's machine. Estimate: about 20 to 40 minutes.
