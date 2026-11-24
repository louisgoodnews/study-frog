# Benchmarking

Tool: `criterion`, in each crate's `benches/`. Profile before optimising (`perf`, `cargo flamegraph`).

Budgets are **provisional**: derived from interaction-time reasoning, not measured. Each is
replaced by a measured baseline when its crate gets its first benchmark (the owning task is listed).

| Hot path | Crate | Provisional budget | Why this number | Measured in |
|---|---|---|---|---|
| Rehearsal loop step | rehearsal | < 10 us per transition (measured 23 ns) | negligible next to a 16.6 ms frame; keeps run logic off the critical path | TASK-0035 (done) |
| FSRS scheduling | srs | 100k cards < 10 ms (was 50 ms) | a full-vault reschedule should not be perceptible; tightened after measuring | TASK-0019 (done) |
| Markdown parse + index | md/store | 10k notes cold index < 3 s (measured 0.57 s) | startup on a large vault stays under a few seconds | TASK-0028 (done) |
| SLM comparison | slm | p95 < 150 ms for 30 words (CPU) | interactive feedback threshold (about 100-200 ms feels immediate to a user) | TASK-0050 |
| Graph layout + projection | graph | 5k nodes, 10k edges: < 8 ms per frame, zero allocation (measured 4.0 ms, 0 allocs) | half of the 16.6 ms frame budget, leaving the rest for painting | TASK-0043 (done) |
| Analytics (EPIC-034): survival, chronotype, fatigue, leeches | report | < 50 ms each on 100k reviews | the reports screen computes them all on open (docs/specs/reports.md) | STORY-0107, 0102-0104 |
| Simulated forecast (EPIC-034) | report | 5,000 cards, 365 days, 16 futures < 50 ms | computed with the reports on open | STORY-0105 |
| Workload against retention (CMRR) | report | 5,000 cards, 12 targets, 180 days < 50 ms | computed with the reports on open | STORY-0106 |
| PDF: structure / all text / one page's words / draw one page, 500 pages | pdf | < 50 ms / < 2 s / < 100 ms / < 50 ms | opening must feel instant, a first scan of a big PDF is a background pass, turning a page and drawing it stay under a tenth of a second | TASK-0122 (done) |

Rules: commit baselines with `--save-baseline`; PRs touching a hot path report the diff;
introduce `smallvec`/`ahash`/`bumpalo` only with a before/after number in the commit message.
Allocation counting is not automated: a counting global allocator needs `unsafe`, which is forbidden workspace-wide. Use `dhat` (heap profiling) when a profile points at the allocator.

## Reference run on the 1.0.0 reference machine (2026-11-24, TASK-0278, STORY-0201, STORY-0203)
Machine (ADR 0014 names it the reference machine; identity in `docs/benchmarks/reference-1.0.0/machine.txt`): Intel Core
i5-10210U @ 1.60 GHz, 8 logical CPUs, Linux 7.3.5, rustc 1.99.1, commit `ec88f3e`. Hygiene not met: governor `powersave`,
turbo enabled, SMT on, 14 processes above 10% CPU, two uncommitted files. Run: `scripts/bench-reference.sh homebox-ref <out> 5`
(each case 5 invocations, 10 when the spread of the 5 exceeds 5%). The 28 medians are in `docs/benchmarks/reference-1.0.0/*.ns`.

| Bench | Median | Spread (runs) | Budget | Status |
|---|---|---|---|---|
| `fsrs_schedule_100k` | 4.86 ms | 2.1% (5) | < 10 ms | within |
| `sm2_schedule_100k` | 1.07 ms | 2.3% (5) | n/a | |
| `vault_10k` scan + parse | 235 ms | 4.9% (5) | n/a | |
| `vault_10k` cold reindex | 847 ms | 1.5% (5) | < 3 s | within |
| `vault_10k` warm reindex | 77.2 ms | 29.6% (10) | n/a | noisy |
| `graph_5k` layout step | 7.00 ms | 2.6% (5) | < 8 ms | within, close to the limit |
| `graph_5k` projection | 2.0 us | 13.3% (10) | n/a | |
| rehearsal, 1000 items (2001 steps) | 72.9 us (about 36 ns per step) | 60.3% (10) | < 10 us per step | within |
| rehearsal queue 10k spaced / shuffled / hardest / lowest | 0.89 / 0.11 / 1.10 / 0.86 ms | 3.4 / 10.0 / 9.1 / 18.5% | n/a | |
| reports: retention / latency / backtest (103k reviews) | 0.70 / 2.13 / 11.0 ms | 3.1 / 3.6 / 16.2% | < 50 ms | within |
| reports: familiarity (10k items) | 8.65 ms | 1.9% (5) | < 20 ms | within |
| analytics: survival / chronotype / fatigue / leeches / confusion | 12.2 / 6.0 / 6.3 / 1.4 / 5.6 ms | 2.4 / 4.4 / 27.9 / 45.4 / 39.8% | < 50 ms | within |
| analytics: forecast, 5,000 cards, 365 days, 16 futures | 39.6 ms | 3.5% (5) | < 50 ms | within |
| analytics: CMRR, 5,000 cards, 12 targets, 180 days | **84.8 ms** | 1.8% (5) | < 50 ms | **over budget on this machine** |
| `pdf_500_pages` info / all text / one page's words / render 1x | 3.3 ms / 653 ms / 92 ms / 9.4 ms | 3.9 / 41.8 / 3.4 / 9.4% | < 50 ms / < 2 s / < 100 ms / < 50 ms | within (words close to limit) |
| `slm_lexical_30` | 21.6 us | 4.7% (5) | semantic p95 < 150 ms (not run here) | n/a |

Spread rule (EPIC-067): 12 of 28 cases are still above 5% after the run (the 12 were extended to 10 invocations): `analytics_leeches` 45.4%,
`rehearsal_run_1000` 60.3%, `pdf_all_text` 41.8%, `analytics_confusion` 39.8%, `vault_warm_reindex` 29.6%, `analytics_fatigue` 27.9%,
`report_backtest` 16.2%, `rehearsal_queue_lowest` 18.5%, `graph_project` 13.3%, `rehearsal_queue_shuffled` 10.0%, `rehearsal_queue_hardest`
9.1%, `pdf_render_1x` 9.4%. The rule does not hold on this machine yet. The budgets are not changed here: the CMRR miss and the noise
are recorded for the owner's decision (STORY-0203). Interleaved paired runs (TASK-0279) are still open.

## Release 1.0.0 check (2026-11-24, TASK-0282, STORY-0202)
Steps (RELEASING step 3, and the regression check before a tag): (1) `cargo bench --workspace` on the reference machine, idle;
(2) `scripts/bench-reference.sh <label> <out> 5` on the release commit; (3) `scripts/bench-compare.sh <baseline.ns> <new.ns>` for each
case; a `regression` verdict blocks the tag unless the owner accepts it in writing.

- `cargo bench --workspace` (commit 76461b6, one busy process): the figures agree with the reference run of the same code
  (`docs/benchmarks/reference-1.0.0/`): FSRS 100k 4.46 ms, SM-2 1.03 ms, graph layout step 6.14 ms, PDF info 3.2 ms, cold reindex 822 ms,
  warm reindex 67 ms, rehearsal run-through 52 us, reports retention 0.60 ms, latency 1.85 ms, familiarity 7.9 ms, CMRR 71 ms (over the
  50 ms budget, as before). No baseline table row changed.
- Release reference run (`docs/benchmarks/release-1.0.0/`, same code, machine state changed after a container restart: 1 busy process,
  load 1.1, against 14 busy processes in the first run): `bench-compare` gives `regression` for 16 of 28 cases with effect sizes of 17%
  to 238% (for example `pdf_info` 3.1x, `pdf_render_1x` 3.4x), and `no-change` for 12. The code under test did not change between the two
  runs, so the verdict is not a code regression, but it is not explained either. Verdict: **regression, not accepted**. The tag is held
  until the owner accepts it in writing or a repeat run clears it.

### Repeat runs (owner decision, 2026-11-24)
Two more full reference runs were taken on the same commit (5b8d8d8 code, `docs/benchmarks/repeat-1.0.0/`):
- Run 3 and run 4 agree with each other: `bench-compare` gives `no-change` for all 28 cases, including every case against run 2.
  Runs 2 to 4 were taken after the container restart, and the machine is now steady at that level.
- Against run 1 (taken before the restart, 14 busy processes) run 4 gives `regression` for 17 of 28 cases (effect +16% to +35%
  typically, `pdf_info` +209%, `pdf_render_1x` +227%) and `no-change` for 11. The `cargo bench --workspace` figures also match
  runs 2 to 4, not run 1. Cause is not established: the machine state changed, and the code under test did not.
- Spread rule after 10 runs (run 4): 5 cases still above 5% (`pdf_render_1x` 13.2%, `pdf_info` 9.2%, `rehearsal_queue_lowest` 9.9%,
  `rehearsal_queue_shuffled` 7.4%, `graph_project` 6.9%). Run 3 had 6 such cases. The rule does not hold yet.
- Verdict: the regressions against run 1 persist after repeated runs. The tag is not made (owner instruction).

## Release 0.1.0 check (2026-10-31)
Run on the same machine (i9-11900K), release profile, each crate's benchmark on its own at low priority (running
`cargo bench --workspace` at once overloaded the machine). Every hot path is within its budget:

| Bench | Result | Budget |
|---|---|---|
| `fsrs_schedule_100k` | 2.86 ms | < 10 ms |
| `sm2_schedule_100k` | 0.69 ms | n/a |
| `vault_10k` scan + parse | 120 ms | n/a |
| `vault_10k` cold reindex | 247 ms | < 3 s |
| `vault_10k` warm reindex | 18.6 ms | n/a |
| `graph_5k` layout step | 4.0 ms | < 8 ms |
| `graph_5k` projection | 1.7 us | n/a |
| comparison latency (language model, release test) | p50 30.2 ms, p95 33.9 ms | p95 < 150 ms |

**0.3.0 release run (2026-11-03, a slower machine than the baselines above):** every bench ran about 2x slower than the
table, including ones whose code did not change (`sm2_schedule_100k` 1.53 ms, was 0.69 ms). Measured: `fsrs_schedule_100k`
6.6 ms (budget < 10 ms), `vault_10k` cold reindex 1.04 s (< 3 s), warm 122 ms, `graph_5k` layout step **8.9 ms (budget
< 8 ms)** and projection 2.7 us, rehearsal queues of 10k items 0.16 to 1.3 ms. The graph budget is missed on this machine
only; `studyfrog-graph` has not changed since M4, so no regression is claimed, but it was not re-measured on the
baseline machine. The language-model latency test was not run (model not installed).

The rehearsal and report benchmarks ran to completion (rehearsal steps 33 us to 570 us per case, reports 0.36 ms to
6.2 ms); their cases are named in the crates' `benches/`.

## Recorded baselines
Machine: Intel i9-11900K @ 3.5 GHz, 16 threads, Linux, rustc 1.99.1, release profile (criterion `--save-baseline m1`).

| Date | Bench | Result | Per item |
|---|---|---|---|
| 2026-10-30 | `fsrs_schedule_100k` (`next_state` + `interval_days`) | 2.93 ms | about 29 ns |
| 2026-10-30 | `sm2_schedule_100k` (`sm2_next`) | 0.72 ms | about 7 ns |

Re-run with `cargo bench -p studyfrog-srs --bench scheduling -- --baseline m1`.

10k-note vault (about 1.5 KB per note, 100 stacks, in-memory DB; `cargo bench -p studyfrog-store --bench vault -- --baseline m2`). Noisy at 10 samples (about +/-10%):

| Date | Bench | Result |
|---|---|---|
| 2026-10-30 | `scan_parse_only` | 282 ms (about 28 us per note) |
| 2026-10-30 | `cold_reindex` (scan + parse + SQL) | 571 ms |
| 2026-10-30 | `warm_reindex_no_changes` (stat 10k files + read index) | 43 ms |

Rehearsal (`cargo bench -p studyfrog-rehearsal --bench rehearsal -- --baseline m3`):

| Date | Bench | Result |
|---|---|---|
| 2026-10-30 | 1000 items flipped and rated (2001 steps) | 46.4 us (about 23 ns per step) |
| 2026-10-30 | `build_queue` 10k candidates, spaced | 337 us |
| 2026-10-30 | `build_queue` 10k candidates, shuffled | 82 us |
| 2026-10-30 | `build_queue` 10k candidates, hardest first | 1.08 ms |
| 2026-10-30 | `build_queue` 10k candidates, lowest retrievability | 3.9 ms |

Graph (`cargo bench -p studyfrog-graph --bench layout -- --baseline m4`; 5,000 nodes, 10,000 edges):

| Date | Bench | Result |
|---|---|---|
| 2026-10-30 | `layout_step` | 3.98 ms |
| 2026-10-30 | `project` | 1.8 us |

Reports (`cargo bench -p studyfrog-report`; a simulated history of 103,596 reviews; budget: each under 50 ms):

| Date | Bench | Result |
|---|---|---|
| 2026-10-30 | retention curve over all reviews | 0.38 ms |
| 2026-10-30 | latency percentiles and histogram, one sample per review | 1.3 ms |
| 2026-10-30 | backtest: replay 103k reviews, estimate the model, forecast, compare | 6.6 ms |

Answer comparison (`cargo bench -p studyfrog-slm`): 21 us per comparison with the lexical runtime; the semantic
runtime is measured by `cargo test --release -p studyfrog-candle -- --nocapture` (p95 about 75 ms, budget 150 ms).

### Profiling pass (M8, 2026-10-30)
Every benchmark of the workspace, run alone on the reference machine (`cargo bench --workspace`):

| Bench | Result | Budget |
|---|---|---|
| rehearsal: 1000 items flipped and rated (2001 steps) | 39 us (about 20 ns per step) | under 10 us per step |
| rehearsal: `build_queue` 10k spaced / shuffled / hardest first | 0.29 ms / 0.08 ms / 0.69 ms | none set |
| rehearsal: `build_queue` 10k lowest recall | 3.39 ms before, **0.47 ms after** | none set |
| srs: FSRS 100k cards / SM-2 100k cards | 3.4 ms / 0.85 ms | under 10 ms |
| store: scan+parse / cold index / warm re-index, 10k notes | 205 ms / 437 ms / 36 ms | cold index under 3 s |
| graph: layout step / projection, 5k nodes | 4.9 ms / 2 us | under 8 ms |
| slm: lexical comparison of 30 words | 15 us | semantic p95 under 150 ms (measured 75 ms) |
| report: retention / latency / backtest, 103k reviews | 0.44 / 1.5 / 8.2 ms | under 50 ms |

All inside their budgets. The one hot spot was the lowest-recall order, whose comparator computed a `powf` twice
per comparison; computing the recall once per item made it 7 times faster. Numbers wobble by 10 to 25 % between
runs on this shared machine (the graph step read 4.0 ms in M4 and 4.9 ms now). `perf` is not installed here, so
the pass used the benchmarks and the structure of the code, not a sampling profile.

Allocation: `crates/studyfrog-graph/tests/allocs.rs` uses `dhat` as the test binary's global allocator and asserts
0 allocations over 50 frames. This is the safe way to count allocations (no `unsafe` in our code).


## PDF support (2026-10-31, TASK-0122)
`cargo bench -p studyfrog-pdf --bench pdf` generates a 500-page file (40 lines of text per page) and measures:

| Bench | Result | Budget |
|---|---|---|
| `info` (page sizes, outline, properties) | 2.1 ms | under 50 ms |
| `all_text` (text of 500 pages, for the index) | 374 ms (0.75 ms per page) | under 2 s |
| `page_text_with_words` (one page with word boxes) | 51.7 ms | under 100 ms |
| `render_one_page_at_1x` | 6.0 ms | under 50 ms |
| `reverse_all_pages` (rewrite all 500 pages in reverse) | 25.5 ms | none set |

`page_text_with_words` is dominated by parsing the whole document again for each call; if turning pages ever feels slow
on very large files, keep the parsed document in the worker. Drawing was measured with the 8 ms median per page found in
the library spike (`docs/research/pdf-library-spike.md`) on real documents.

Familiarity (`cargo bench -p studyfrog-report --bench reports -- familiarity`; 10 000 items with 20 ratings and 5 views each; budget: 20 ms): 7.3 ms (2026-11-02).

## Release 0.5.0 check (2026-11-08)
Each crate's benchmark on its own at low priority, release profile, on the development machine (not otherwise idle: a
build and the gate had just run). Every hot path is within its budget:

| Bench | Result | Budget |
|---|---|---|
| `fsrs_schedule_100k` | 3.19 ms | < 10 ms |
| `sm2_schedule_100k` | 0.80 ms | n/a |
| `vault_10k` scan + parse | 181 ms | n/a |
| `vault_10k` cold reindex | 564 ms | < 3 s |
| `vault_10k` warm reindex | 53 ms | n/a |
| `graph_5k` layout step | 4.13 ms | < 8 ms |
| `graph_5k` projection | 1.8 us | n/a |
| rehearsal queues of 10k items | 0.08 to 0.62 ms | n/a |
| PDF, 500 pages: structure / all text / one page's words / draw one page | 1.9 ms / 360 ms / 52 ms / 5.8 ms | < 50 ms / < 2 s / < 100 ms / < 50 ms |
| PDF: draw one page at 2x / one thumbnail / parse the document only | 8.3 ms / 3.1 ms / 0.59 ms | n/a |
| comparison latency (language model, release test) | p50 28.9 ms, p95 31.0 ms | p95 < 150 ms |

The last three PDF rows were added for STORY-0085: re-parsing the document on every render costs about a tenth of a
render, so keeping a parsed document between renders was not built.


## Analytics (EPIC-034)
`cargo bench -p studyfrog-report --bench analytics` (simulated history of 103,596 reviews: 400 days, 40 new items a day,
seed 1; release profile, this machine, criterion with 3 s measurement):

| Date | Bench | Result | Budget |
|---|---|---|---|
| 2026-11-08 | survival report (replay, tiers, fit) | 5.2 ms | < 50 ms |
| 2026-11-08 | chronotype of every review and answer time | 3.3 ms | < 50 ms |
| 2026-11-08 | fatigue report of one sample per review | 2.8 ms | < 50 ms |
| 2026-11-08 | leeches of every item (16,000 items) | 0.7 ms | < 50 ms |
| 2026-11-08 | confusion pairs of 400 lapsing items | 3.0 ms | < 50 ms |
| 2026-11-08 | forecast of 5,000 cards, 90 days, 16 futures | 7.9 ms | < 50 ms |
| 2026-11-08 | forecast of 5,000 cards, 365 days, 16 futures | 21.0 ms | < 50 ms |
| 2026-11-08 | cmrr of 5,000 cards, 12 targets, 180 days, 4 futures | 45.4 ms | < 50 ms |

## Release 0.6.0 check (2026-11-09)
Each crate's benchmark on its own at low priority, release profile, on the development machine. The first pass ran while
the machine was busy (load average about 9, another project compiling); the one number that missed its budget there was run
again alone (see below). Every hot path is within its budget:

| Bench | Result | Budget |
|---|---|---|
| `fsrs_schedule_100k` | 2.71 ms | < 10 ms |
| `sm2_schedule_100k` | 0.70 ms | n/a |
| `vault_10k` scan + parse / cold reindex / warm reindex | 118 ms / 384 ms / 35 ms | cold < 3 s |
| `graph_5k` layout step / projection | 3.63 ms / 1.6 us | < 8 ms |
| rehearsal queues of 10k items | 0.06 to 0.56 ms | n/a |
| PDF, 500 pages: structure / all text / one page's words / draw one page | 5.9 ms / 300 ms / 45 ms / 8.1 ms | < 50 ms / < 2 s / < 100 ms / < 50 ms |
| PDF: draw at 2x / one thumbnail / parse the document only | 9.5 ms / 7.8 ms / 0.52 ms | n/a |
| survival report (true retention, tier curves, fit) | 6.7 ms | see the analytics budgets above |
| chronotype / fatigue report | 3.3 ms / 2.7 ms | see the analytics budgets above |
| leeches of every item / confusion pairs of 400 lapsing items | 0.69 ms / 3.0 ms | see the analytics budgets above |
| forecast of 5,000 cards: 90 days / 365 days, 16 futures | 8.0 ms / 21.0 ms | see the analytics budgets above |
| workload against retention (CMRR), 5,000 cards, 12 targets, 180 days | 45.2 to 45.5 ms alone (57.7 ms in the busy first pass) | < 50 ms |
| comparison latency (language model, release test) | p50 24.4 ms, p95 24.7 ms | p95 < 150 ms |

The CMRR simulation is the closest to its budget (about 10 % under it on a quiet machine, over it on a loaded one); it
runs once when the Reports screen is built, so a loaded machine makes the report slower to appear, not the app slower.

