---
id: STORY-0201
type: story
title: "Reference runs on the owner's machine, with the machine's identity recorded"
status: in_progress
epic: EPIC-067
parents: ['EPIC-067']
estimate: 5
labels: [m14, perf, tooling]
acceptance_criteria:
  - 'Each budgeted benchmark runs at least 5 times, as separate process invocations, on the owner''s machine, and every median is saved'
  - 'Every saved result carries the machine identity, the commit and the hygiene state of its run'
  - 'A case whose 5 medians spread more than 5% is repeated to 10 runs before its result is used'
  - 'Two builds can run interleaved (baseline and new, alternating) so that the comparison is paired'
---

## Context
EPIC-067 Decisions: benchmarks run on the owner's local machine on a schedule, baselines are per machine,
and each side gets 5 interleaved invocations (10 when the spread is above 5%). The 0.3.0 run showed a 2x gap
between machines with no code change, so a result without its machine is not comparable.

## Description
- `scripts/bench-machine.sh` writes the machine identity (CPU model, logical CPUs, kernel, rustc, commit, dirty
  files) and the hygiene state (governor, boost, SMT, load, busy processes), and warns for each item not met.
- `scripts/bench-reference.sh` runs every budgeted case of BENCHMARKING.md, N times, each invocation a separate
  process, and writes one median per invocation (ns) to `<out>/<case>.ns`. It extends a noisy case to 10 runs.
- Interleaving two builds is a separate task, because it needs two built binaries.

## Acceptance Criteria
- [ ] Each budgeted benchmark runs at least 5 times on the owner's machine, and every median is saved (the script is ready; the run needs that machine)
- [x] Every saved result carries the machine identity, the commit and the hygiene state of its run (TASK-0277)
- [x] A case whose 5 medians spread more than 5% is repeated to 10 runs (TASK-0278)
- [ ] Two builds run interleaved, so the comparison is paired (TASK-0279)

## Design Notes
- The script runs on the same machine as the baseline only. It never compares across machines.
- Criterion baselines are saved as `<label>-run<i>`, so criterion's own reports can read any run later.
- Hostname and user name are not recorded; the owner gives the machine a label.
- The container used to build this has no cpufreq or boost controls, so it is not a reference machine.
  A test run there spread 9.8% over 5 runs and was extended to 10 (see TASK-0278).

## Dependencies
TASK-0277, TASK-0278, TASK-0279. The owner's machine for the reference runs (the blocker for the first and last
acceptance criteria). STORY-0202 uses the output.

## Open Questions
- The idle run-to-run spread of the owner's machine (EPIC-067 Open Questions) is not measured yet.
