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

Rules: commit baselines with `--save-baseline`; PRs touching a hot path report the diff;
introduce `smallvec`/`ahash`/`bumpalo` only with a before/after number in the commit message.
Allocation counting is not automated: a counting global allocator needs `unsafe`, which is forbidden workspace-wide. Use `dhat` (heap profiling) when a profile points at the allocator.

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

