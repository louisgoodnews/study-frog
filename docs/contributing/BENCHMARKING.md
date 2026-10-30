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

Allocation: `crates/studyfrog-graph/tests/allocs.rs` uses `dhat` as the test binary's global allocator and asserts
0 allocations over 50 frames. This is the safe way to count allocations (no `unsafe` in our code).

