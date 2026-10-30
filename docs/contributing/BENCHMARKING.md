# Benchmarking

Tool: `criterion`, in each crate's `benches/`. Profile before optimising (`perf`, `cargo flamegraph`).

| Hot path | Crate | Metric / budget (initial, revisit with data) |
|---|---|---|
| Rehearsal loop step | rehearsal | < 10 us per transition |
| FSRS scheduling | srs | 100k cards < 50 ms |
| Markdown parse + index | md/store | 10k notes cold scan < 3 s |
| SLM comparison | slm | p95 < 150 ms for 30 words (CPU) |
| Graph layout/render | ui | 5k nodes at 60 fps, zero allocation per frame |

Rules: commit baselines with `--save-baseline`; PRs touching a hot path report the diff;
introduce `smallvec`/`ahash`/`bumpalo` only with a before/after number in the commit message.
Allocation counts are checked with a counting global allocator in bench builds.
