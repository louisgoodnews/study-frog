# ADR 0004: SLM runtime

- Status: **Accepted** (2026-10-30); the M6 spike (TASK-0050) must still confirm the latency budget
- Date: 2026-10-30

## Context
Compare user text to reference text and return a score, locally, quickly (interactive).
The runtime must be swappable behind a trait.

## Options
| | candle | llama.cpp bindings | ort (ONNX Runtime) |
|---|---|---|---|
| Language/build | pure Rust | C++ build, FFI | prebuilt native lib, FFI |
| Embedding models (MiniLM-class) | yes | possible, less natural | yes, mature |
| Generative judge (small LLM, GGUF) | yes (quantised) | best support | via exported ONNX, awkward |
| Build/deploy simplicity | high | medium | medium (library distribution) |
| `unsafe_code = "forbid"` friendliness | in our code, yes | crate wraps FFI | crate wraps FFI |

## Decision (proposed)
**candle** first, embedding-based similarity as the primary score (fast, deterministic,
cheap), with an optional generative-judge path later. The trait:

```rust
trait SlmRuntime {
    fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, SlmError>;
}
```
Scoring (cosine plus rubric adjustments, [scoring](../specs/scoring.md)) is pure free functions
over embeddings, so a runtime swap does not change scoring logic.

## Spike (M6, TASK-0050)
Measure p50/p95 latency for a 30-word comparison, memory, and model download size on a
reference machine; compare against `ort` with the same model. If candle misses the budget
(p95 > 150 ms CPU) switch to `ort`; only `studyfrog-slm` changes.

## Progress (M6)
The trait, the scoring functions and an in-house lexical runtime are implemented and benchmarked (about
21 µs per comparison, so the budget is met trivially, but with no semantic understanding). The candle spike
(TASK-0050) has **not** been run: it needs a model download, so the numbers for candle vs `ort` are still
missing and this ADR stays Accepted-but-unconfirmed on that point.

## Consequences
Models are downloaded/placed by the user into the app data dir; nothing is bundled.
