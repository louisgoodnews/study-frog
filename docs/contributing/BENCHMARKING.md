# Benchmarking

Tool: `criterion`, in each crate's `benches/`. Profile before optimising (`perf`, `cargo flamegraph`).

Budgets are **provisional**: derived from interaction-time reasoning, not measured. Each is
replaced by a measured baseline when its crate gets its first benchmark (the owning task is listed).

| Hot path | Crate | Provisional budget | Why this number | Measured in |
|---|---|---|---|---|
| Rehearsal loop step | rehearsal | < 10 us per transition | negligible next to a 16.6 ms frame; keeps run logic off the critical path | TASK-0035 |
| FSRS scheduling | srs | 100k cards < 10 ms (was 50 ms) | a full-vault reschedule should not be perceptible; tightened after measuring | TASK-0019 (done) |
| Markdown parse + index | md/store | 10k notes cold index < 3 s (measured 0.57 s) | startup on a large vault stays under a few seconds | TASK-0028 (done) |
| SLM comparison | slm | p95 < 150 ms for 30 words (CPU) | interactive feedback threshold (about 100-200 ms feels immediate to a user) | TASK-0050 |
| Graph layout/render | ui | 5k nodes at 60 fps, zero allocation per frame | 60 fps = 16.6 ms frame budget | TASK-0043 |

Rules: commit baselines with `--save-baseline`; PRs touching a hot path report the diff;
introduce `smallvec`/`ahash`/`bumpalo` only with a before/after number in the commit message.
Allocation counts are checked with a counting global allocator in bench builds.

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
