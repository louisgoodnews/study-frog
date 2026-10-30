# Benchmarking

Tool: `criterion`, in each crate's `benches/`. Profile before optimising (`perf`, `cargo flamegraph`).

Budgets are **provisional**: derived from interaction-time reasoning, not measured. Each is
replaced by a measured baseline when its crate gets its first benchmark (the owning task is listed).

| Hot path | Crate | Provisional budget | Why this number | Measured in |
|---|---|---|---|---|
| Rehearsal loop step | rehearsal | < 10 us per transition | negligible next to a 16.6 ms frame; keeps run logic off the critical path | TASK-0035 |
| FSRS scheduling | srs | 100k cards < 50 ms | a full-vault reschedule should not be perceptible | TASK-0019 |
| Markdown parse + index | md/store | 10k notes cold scan < 3 s | startup on a large vault stays under a few seconds | TASK-0028 |
| SLM comparison | slm | p95 < 150 ms for 30 words (CPU) | interactive feedback threshold (about 100-200 ms feels immediate to a user) | TASK-0050 |
| Graph layout/render | ui | 5k nodes at 60 fps, zero allocation per frame | 60 fps = 16.6 ms frame budget | TASK-0043 |

Rules: commit baselines with `--save-baseline`; PRs touching a hot path report the diff;
introduce `smallvec`/`ahash`/`bumpalo` only with a before/after number in the commit message.
Allocation counts are checked with a counting global allocator in bench builds.
