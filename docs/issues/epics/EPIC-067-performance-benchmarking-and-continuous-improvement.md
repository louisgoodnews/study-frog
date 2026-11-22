---
id: EPIC-067
type: epic
title: "Performance benchmarking and continuous improvement"
status: in_progress
parents: []
labels: [m14, perf, tooling]
acceptance_criteria:
  - 'Every hot path in docs/contributing/BENCHMARKING.md has a benchmark, a recorded baseline on a named machine, and a budget that the baseline supports'
  - 'A change that slows a hot path past its budget or past its baseline by more than the agreed margin is caught before it is released'
  - 'Frame time and allocations of the interactive screens are measured, not guessed, and the slowest of them has a profile on record'
---

## Context
The budgets in `docs/contributing/BENCHMARKING.md` were partly derived from reasoning, not measurement, and they are marked provisional. The 0.3.0 run showed the same benchmarks about 2x slower on a different machine, and the graph layout budget (< 8 ms) was missed there with no code change to explain it. Nothing runs the benchmarks automatically, so a regression is found only when someone happens to run them.

## Description
Make performance a routine check instead of an occasional one:

- Measure every hot path that has a budget and replace each provisional number with a measured baseline, recorded with the machine it came from.
- Run the benchmarks on one fixed machine or runner, so a baseline means the same thing from release to release. Record the machine's identity with every result.
- Compare each run with the saved baseline and report the change, so a regression shows as a number in a pull request or a release note.
- Cover what the benchmarks do not yet reach: the frame time of the rehearsal card, the reports screen and the console output on a large vault, and the allocations in render loops (`dhat`, since a counting allocator would need `unsafe`).
- Profile before optimising: each slow path found gets a profile recorded in its task, and a change that claims a speed-up must show the before and after numbers in the commit message (rule P7).
- Keep the improvement loop small: a finding becomes a task with its budget, its baseline and its measured result, and is closed only when the number is back inside the budget.

## Acceptance Criteria
- [ ] Every hot path in `docs/contributing/BENCHMARKING.md` has a benchmark, a baseline on a named machine, and a budget the baseline supports
- [ ] A change that slows a hot path past its budget or past its baseline by the agreed margin is caught before release
- [ ] Frame time and allocations of the interactive screens are measured, and the slowest has a profile on record

## Design Notes
- Benchmarks stay in each crate's `benches/` with `criterion`; the comparison and reporting are a script under `scripts/`, not new crates.
- Budgets stay provisional until a baseline exists; a budget that a baseline cannot support is changed in the document, with the reason.
- The gate (`scripts/check.sh`) stays fast: the benchmarks run on demand or on a schedule, not in every gate run.
- Machine variance is handled by comparing a run against the baseline of the same machine only, never across machines.

## Dependencies
The existing benchmarks in the eight crates that have them (`studyfrog-srs`, `studyfrog-store`, `studyfrog-graph`, `studyfrog-rehearsal`, `studyfrog-report`, `studyfrog-slm`, `studyfrog-pdf`). The owner's local machine, on a schedule (see Decisions).

## Decisions
- **Where and when:** benchmarks run on the owner's local machine, on a schedule. Baselines are per machine and never compared across machines.
- **Frame time:** measured in the running window, not in the egui test harness.
- **Regression rule:** flag a regression only when both hold: (a) the bootstrap 95% confidence interval of the new/baseline ratio of medians excludes 1.0, and (b) the point estimate exceeds an effect-size floor of max(5%, 3 times the baseline's own run-to-run spread). The 5% floor is a judgment, not a published value; published sources support statistical gating with a noise-derived floor but no universal percentage.
- **Repeats:** at least 5 interleaved process invocations per side (baseline and new alternating), each with criterion's default sample count; 10 if the spread across the 5 exceeds 5%. Interleaving allows a paired comparison.
- **Baselines:** one per machine. The 2x gap seen between machines is a machine-identity problem that no margin fixes.
- **Machine hygiene:** performance governor, boost disabled, pinned cores (taskset), SMT off where possible, background load closed (Google Benchmark, "Reducing Variance").

## Open Questions
- Measure the i9-11900K's own idle run-to-run spread (10 back-to-back runs). That sets the noise floor in rule (b); until then the 5% floor is provisional.
- Confirm the noise threshold of the criterion version in use; the sources gave different defaults (1% and 2%).
- Sources consulted: criterion.rs user guide; Georges et al. (OOPSLA 2007 and 2008) on confidence intervals and paired comparison; Kalibera and Jones (ISMM 2013) on repetitions; Daly et al. (MongoDB, E-divisive change points); Hunter (DataStax, ICPE 2023); Chromium and LLVM regression policies (thresholds of 0.3% to 2%, for competitive benchmarks, not a general default).
